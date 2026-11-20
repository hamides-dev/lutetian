#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod platform;
mod ui;

use iced::futures::SinkExt;
use iced::widget::container;
use iced::{Element, Size, Subscription, Task, Theme, stream, window};
use lutetian_bridge::{BridgeEvent, BridgeState, RunningBridge, start_bridge_server};
use lutetian_core::{
    ActiveBridgeInfo, ActivityLevel, AppConfig, AppStorage, LutetianProject, PipelineStage,
    UpdateCheckResult, UpdaterClient, export_engine_bundle, has_internet_connectivity,
};
use platform::updater_exec::launch_installer;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use ui::blueprint_canvas::CanvasMsg;
use ui::dashboard::{DashboardMsg, DashboardState, DashboardTab, view_dashboard};
use ui::icons::load_app_icon_rgba;
use ui::theme::transparent_iced_theme_for;

#[derive(Debug, Clone)]
enum AppBridgeMsg {
    Started { port: u16, state: BridgeState },
    Event(BridgeEvent),
    Failed(String),
    WakeWindow,
}

static BRIDGE_UI_RX: OnceLock<Mutex<Option<UnboundedReceiver<AppBridgeMsg>>>> = OnceLock::new();

#[cfg(target_os = "windows")]
static WAKE_EVENT_HANDLE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[cfg(target_os = "windows")]
fn acquire_windows_mutex_or_wake_existing() {
    use windows_sys::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError};
    use windows_sys::Win32::System::Threading::{
        CreateEventW, CreateMutexW, EVENT_MODIFY_STATE, OpenEventW, SetEvent,
    };

    let mutex_name: Vec<u16> = "LutetianStudioSingleInstanceMutex\0"
        .encode_utf16()
        .collect();
    let event_name: Vec<u16> = "Local\\LutetianWakeWindowEvent\0".encode_utf16().collect();

    unsafe {
        let mutex = CreateMutexW(std::ptr::null(), 0, mutex_name.as_ptr());
        let err = GetLastError();
        if mutex.is_null() || err == ERROR_ALREADY_EXISTS {
            let ev = OpenEventW(EVENT_MODIFY_STATE, 0, event_name.as_ptr());
            if !ev.is_null() {
                let _ = SetEvent(ev);
                let _ = CloseHandle(ev);
            }
            if !mutex.is_null() {
                let _ = CloseHandle(mutex);
            }
            std::process::exit(0);
        }

        let ev = CreateEventW(std::ptr::null(), 0, 0, event_name.as_ptr());
        if !ev.is_null() {
            WAKE_EVENT_HANDLE.store(ev as usize, std::sync::atomic::Ordering::SeqCst);
        }
    }
}

fn main() -> iced::Result {
    #[cfg(target_os = "windows")]
    acquire_windows_mutex_or_wake_existing();

    let mut daemon = iced::daemon(LutetianApp::new, LutetianApp::update, LutetianApp::view)
        .title(LutetianApp::title)
        .theme(LutetianApp::theme)
        .subscription(LutetianApp::subscription)
        .default_font(ui::theme::font_ui_default());

    for font_bytes in ui::theme::bundled_fonts() {
        daemon = daemon.font(font_bytes);
    }

    daemon.run()
}

struct LutetianApp {
    storage: AppStorage,
    config: AppConfig,
    updater: UpdaterClient,
    project_path: PathBuf,
    project_snapshot: LutetianProject,
    shared_project: Arc<RwLock<LutetianProject>>,
    bridge_state: Option<BridgeState>,
    _bridge_runtime_guard: Arc<Mutex<Option<RunningBridge>>>,

    app_icon: Option<window::Icon>,
    dashboard_id: Option<window::Id>,
    window_size: Size,
    dashboard: DashboardState,
}

#[derive(Debug, Clone)]
enum Message {
    Bridge(AppBridgeMsg),
    WindowOpened(window::Id),
    WindowEventOccurred(window::Id, window::Event),
    Dashboard(DashboardMsg),
    UpdateCheckFinished(Result<UpdateCheckResult, String>),
    AutoUpdateCheckFinished(Option<Result<UpdateCheckResult, String>>),
    PeriodicUpdateCheck,
    UpdateDownloadProgress(Option<u8>),
    UpdateDownloadFinished(Result<PathBuf, String>),
}

impl LutetianApp {
    fn new() -> (Self, Task<Message>) {
        let storage = AppStorage::open_default();
        let config = storage.load_config();
        let project_path = config
            .last_project_path
            .as_ref()
            .map(PathBuf::from)
            .filter(|p| p.exists())
            .unwrap_or_else(|| storage.default_project_path());

        let mut project = storage
            .load_project(&project_path)
            .unwrap_or_else(|_| LutetianProject::create_demo_project());
        project.recompute_validation();
        let _ = storage.save_project(&project_path, &project);

        let shared_project = Arc::new(RwLock::new(project.clone()));
        let (app_tx, app_rx) = unbounded_channel::<AppBridgeMsg>();
        let _ = BRIDGE_UI_RX.set(Mutex::new(Some(app_rx)));

        #[cfg(target_os = "windows")]
        {
            let wake_tx = app_tx.clone();
            std::thread::spawn(move || {
                use windows_sys::Win32::Foundation::WAIT_OBJECT_0;
                use windows_sys::Win32::System::Threading::{INFINITE, WaitForSingleObject};
                let raw = WAKE_EVENT_HANDLE.load(std::sync::atomic::Ordering::SeqCst);
                if raw != 0 {
                    let ev = raw as windows_sys::Win32::Foundation::HANDLE;
                    loop {
                        let res = unsafe { WaitForSingleObject(ev, INFINITE) };
                        if res == WAIT_OBJECT_0 {
                            if wake_tx.send(AppBridgeMsg::WakeWindow).is_err() {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
            });
        }

        // Start embedded HTTP/WebSocket bridge in a dedicated Tokio thread
        let bridge_guard: Arc<Mutex<Option<RunningBridge>>> = Arc::new(Mutex::new(None));
        let bridge_guard_bg = Arc::clone(&bridge_guard);
        let bridge_proj = Arc::clone(&shared_project);
        let bridge_storage = storage.clone();
        let preferred_port = config.bridge_port;
        let bridge_app_tx = app_tx.clone();

        std::thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(err) => {
                    let _ = bridge_app_tx.send(AppBridgeMsg::Failed(err.to_string()));
                    return;
                }
            };
            rt.block_on(async move {
                let (ev_tx, mut ev_rx) = unbounded_channel::<BridgeEvent>();
                match start_bridge_server(preferred_port, bridge_proj, ev_tx).await {
                    Ok(running) => {
                        let port = running.port;
                        let st = running.state.clone();
                        let _ = bridge_storage.write_bridge_info(&ActiveBridgeInfo {
                            host: "127.0.0.1".to_string(),
                            port,
                            pid: std::process::id(),
                        });
                        if let Ok(mut g) = bridge_guard_bg.lock() {
                            *g = Some(running);
                        }
                        let _ = bridge_app_tx.send(AppBridgeMsg::Started { port, state: st });
                        while let Some(ev) = ev_rx.recv().await {
                            if bridge_app_tx.send(AppBridgeMsg::Event(ev)).is_err() {
                                break;
                            }
                        }
                    }
                    Err(err) => {
                        let _ = bridge_app_tx.send(AppBridgeMsg::Failed(err.to_string()));
                    }
                }
            });
        });

        let app_icon =
            load_app_icon_rgba().and_then(|(rgba, w, h)| window::icon::from_rgba(rgba, w, h).ok());
        let (dash_id, open_task) = open_dashboard_task(app_icon.clone());
        let dashboard = DashboardState::new(&config, &project, config.bridge_port);

        let app = Self {
            storage,
            config,
            updater: UpdaterClient::new(),
            project_path,
            project_snapshot: project,
            shared_project,
            bridge_state: None,
            _bridge_runtime_guard: bridge_guard,
            app_icon,
            dashboard_id: Some(dash_id),
            window_size: Size::new(1180.0, 760.0),
            dashboard,
        };

        let mut tasks = vec![open_task];
        if app.config.auto_check_updates {
            tasks.push(app.spawn_auto_update_check());
        }

        (app, Task::batch(tasks))
    }

    fn title(&self, _window: window::Id) -> String {
        format!("Lutetian — {}", self.project_snapshot.name)
    }

    fn theme(&self, _window: window::Id) -> Theme {
        transparent_iced_theme_for(self.config.theme)
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![
            Subscription::run(bridge_stream).map(Message::Bridge),
            window::events().map(|(id, ev)| Message::WindowEventOccurred(id, ev)),
        ];
        if self.config.auto_check_updates {
            subs.push(
                iced::time::every(Duration::from_secs(5 * 60))
                    .map(|_| Message::PeriodicUpdateCheck),
            );
        }
        Subscription::batch(subs)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowOpened(id) => {
                if self.dashboard_id != Some(id) {
                    return window::close(id);
                }
                Task::none()
            }
            Message::WindowEventOccurred(id, event) => match event {
                window::Event::CloseRequested => self.exit_cleanly(id),
                window::Event::Resized(size) => {
                    self.window_size = size;
                    Task::none()
                }
                _ => Task::none(),
            },
            Message::Bridge(msg) => match msg {
                AppBridgeMsg::Started { port, state } => {
                    self.dashboard.bridge_port = port;
                    self.bridge_state = Some(state);
                    Task::none()
                }
                AppBridgeMsg::Event(ev) => {
                    if let Ok(guard) = self.shared_project.try_read() {
                        self.project_snapshot = guard.clone();
                        self.dashboard.sync_from_project(&self.project_snapshot);
                        let _ = self
                            .storage
                            .save_project(&self.project_path, &self.project_snapshot);
                    }
                    match ev {
                        BridgeEvent::ProjectMutated { reason, .. } => {
                            self.dashboard.status_toast = Some(reason);
                        }
                        BridgeEvent::TelemetryUpdated { action } => {
                            self.dashboard.status_toast = Some(action);
                        }
                        BridgeEvent::StageDecisionChanged { notes, .. } => {
                            if !notes.is_empty() {
                                self.dashboard.status_toast = Some(notes);
                            }
                        }
                    }
                    Task::none()
                }
                AppBridgeMsg::Failed(err) => {
                    self.dashboard.error_banner = Some(format!("Bridge error: {err}"));
                    Task::none()
                }
                AppBridgeMsg::WakeWindow => {
                    if let Some(id) = self.dashboard_id {
                        Task::batch([window::minimize(id, false), window::gain_focus(id)])
                    } else {
                        let (id, task) = open_dashboard_task(self.app_icon.clone());
                        self.dashboard_id = Some(id);
                        task
                    }
                }
            },
            Message::Dashboard(msg) => self.update_dashboard(msg),
            Message::UpdateCheckFinished(res) => {
                self.dashboard.update_checking = false;
                match res {
                    Ok(r) => {
                        self.dashboard.update_error = None;
                        self.dashboard.update_status = Some(r);
                    }
                    Err(e) => {
                        self.dashboard.update_error = Some(e);
                    }
                }
                Task::none()
            }
            Message::AutoUpdateCheckFinished(opt) => {
                self.dashboard.update_checking = false;
                if let Some(Ok(r)) = opt {
                    self.dashboard.update_error = None;
                    self.dashboard.update_status = Some(r);
                }
                Task::none()
            }
            Message::PeriodicUpdateCheck => {
                if self.config.auto_check_updates && !self.dashboard.update_checking {
                    self.spawn_auto_update_check()
                } else {
                    Task::none()
                }
            }
            Message::UpdateDownloadProgress(pct) => {
                self.dashboard.update_download_progress = pct;
                Task::none()
            }
            Message::UpdateDownloadFinished(res) => {
                self.dashboard.update_downloading = false;
                self.dashboard.update_download_progress = None;
                match res {
                    Ok(path) => match launch_installer(&path) {
                        Ok(should_exit) => {
                            if should_exit {
                                self.storage.clear_bridge_info();
                                return iced::exit();
                            }
                            self.dashboard.status_toast = Some("Installer launched.".to_string());
                        }
                        Err(e) => self.dashboard.update_error = Some(e),
                    },
                    Err(e) => self.dashboard.update_error = Some(e),
                }
                Task::none()
            }
        }
    }

    fn view(&self, _window: window::Id) -> Element<'_, Message> {
        let el: Element<'_, DashboardMsg> = container(view_dashboard(
            &self.dashboard,
            &self.config,
            &self.project_snapshot,
        ))
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into();
        el.map(Message::Dashboard)
    }

    fn exit_cleanly(&mut self, _id: window::Id) -> Task<Message> {
        let _ = self
            .storage
            .save_project(&self.project_path, &self.project_snapshot);
        let _ = self.storage.save_config(&self.config);
        self.storage.clear_bridge_info();
        iced::exit()
    }

    fn sync_project_and_save(&mut self) {
        self.project_snapshot.recompute_validation();
        let _ = self
            .storage
            .save_project(&self.project_path, &self.project_snapshot);
        if let Ok(mut guard) = self.shared_project.try_write() {
            *guard = self.project_snapshot.clone();
        }
    }

    fn update_dashboard(&mut self, msg: DashboardMsg) -> Task<Message> {
        match msg {
            DashboardMsg::DragTitlebar => {
                if let Some(id) = self.dashboard_id {
                    window::drag(id)
                } else {
                    Task::none()
                }
            }
            DashboardMsg::MinimizeWindow => {
                if let Some(id) = self.dashboard_id {
                    window::minimize(id, true)
                } else {
                    Task::none()
                }
            }
            DashboardMsg::ToggleMaximize => {
                if let Some(id) = self.dashboard_id {
                    self.dashboard.is_maximized = !self.dashboard.is_maximized;
                    window::maximize(id, self.dashboard.is_maximized)
                } else {
                    Task::none()
                }
            }
            DashboardMsg::CloseWindow => {
                if let Some(id) = self.dashboard_id {
                    self.exit_cleanly(id)
                } else {
                    self.storage.clear_bridge_info();
                    iced::exit()
                }
            }
            DashboardMsg::TitlebarCloseHovered(h) => {
                self.dashboard.titlebar_close_hovered = h;
                Task::none()
            }
            DashboardMsg::ResizeWindow(dir) => {
                if let Some(id) = self.dashboard_id {
                    window::drag_resize(id, dir)
                } else {
                    Task::none()
                }
            }

            DashboardMsg::SwitchTab(tab) => {
                self.dashboard.active_tab = tab;
                Task::none()
            }
            DashboardMsg::ToggleSidebar => {
                self.dashboard.sidebar_collapsed = !self.dashboard.sidebar_collapsed;
                Task::none()
            }
            DashboardMsg::SwitchSettingsSubTab(sub) => {
                self.dashboard.settings_subtab = sub;
                Task::none()
            }
            DashboardMsg::DismissToast => {
                self.dashboard.status_toast = None;
                self.dashboard.error_banner = None;
                Task::none()
            }

            DashboardMsg::NewEmptyProject => {
                self.project_snapshot = LutetianProject::new_empty("New Environment Project");
                self.dashboard.sync_from_project(&self.project_snapshot);
                self.dashboard.active_tab = DashboardTab::Stage1Bible;
                self.dashboard.status_toast =
                    Some("Started new empty Stage 1 project.".to_string());
                self.sync_project_and_save();
                Task::none()
            }
            DashboardMsg::LoadDemoProject => {
                self.project_snapshot = LutetianProject::create_demo_project();
                self.dashboard.sync_from_project(&self.project_snapshot);
                self.dashboard.canvas_state.fit_world(
                    self.project_snapshot.blueprint.world_width_m,
                    self.project_snapshot.blueprint.world_depth_m,
                    740.0,
                    480.0,
                );
                self.dashboard.status_toast =
                    Some("Loaded Courtyard Market District sample.".to_string());
                self.sync_project_and_save();
                Task::none()
            }
            DashboardMsg::OpenProjectFile => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Lutetian Project", &["json"])
                    .pick_file()
                {
                    match self.storage.load_project(&path) {
                        Ok(proj) => {
                            self.project_path = path.clone();
                            self.config.last_project_path =
                                Some(path.to_string_lossy().to_string());
                            let _ = self.storage.save_config(&self.config);
                            self.project_snapshot = proj;
                            self.dashboard.sync_from_project(&self.project_snapshot);
                            self.dashboard.canvas_state.fit_world(
                                self.project_snapshot.blueprint.world_width_m,
                                self.project_snapshot.blueprint.world_depth_m,
                                740.0,
                                480.0,
                            );
                            self.dashboard.status_toast =
                                Some(format!("Opened {}", path.display()));
                            self.sync_project_and_save();
                        }
                        Err(e) => {
                            self.dashboard.error_banner = Some(format!("Failed to open: {e}"));
                        }
                    }
                }
                Task::none()
            }
            DashboardMsg::SaveProjectFile => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Lutetian Project", &["json"])
                    .set_file_name("project.lutetian.json")
                    .save_file()
                {
                    self.project_path = path.clone();
                    self.config.last_project_path = Some(path.to_string_lossy().to_string());
                    let _ = self.storage.save_config(&self.config);
                    self.sync_project_and_save();
                    self.dashboard.status_toast = Some(format!("Saved to {}", path.display()));
                } else {
                    self.sync_project_and_save();
                    self.dashboard.status_toast =
                        Some(format!("Saved {}", self.project_path.display()));
                }
                Task::none()
            }

            // Stage 1
            DashboardMsg::BibleTitleChanged(v) => {
                self.project_snapshot.name = v.clone();
                self.project_snapshot.bible.title = v;
                self.sync_project_and_save();
                Task::none()
            }
            DashboardMsg::BibleArtStyleChanged(v) => {
                self.project_snapshot.bible.art_style = v;
                self.sync_project_and_save();
                Task::none()
            }
            DashboardMsg::BibleMoodChanged(v) => {
                self.project_snapshot.bible.mood = v;
                self.sync_project_and_save();
                Task::none()
            }
            DashboardMsg::BibleLightingChanged(v) => {
                self.project_snapshot.bible.lighting = v;
                self.sync_project_and_save();
                Task::none()
            }
            DashboardMsg::BiblePaletteChanged(v) => {
                self.project_snapshot.bible.palette_notes = v;
                self.sync_project_and_save();
                Task::none()
            }
            DashboardMsg::AnchorHumanChanged(v) => {
                self.dashboard.anchor_human_buf = v.clone();
                if let Ok(val) = v.trim().parse::<f64>()
                    && val > 0.0
                {
                    self.project_snapshot.bible.scale_anchors.human_height_m = val;
                    self.sync_project_and_save();
                }
                Task::none()
            }
            DashboardMsg::AnchorDoorHeightChanged(v) => {
                self.dashboard.anchor_door_h_buf = v.clone();
                if let Ok(val) = v.trim().parse::<f64>()
                    && val > 0.0
                {
                    self.project_snapshot.bible.scale_anchors.door_height_m = val;
                    self.sync_project_and_save();
                }
                Task::none()
            }
            DashboardMsg::AnchorDoorWidthChanged(v) => {
                self.dashboard.anchor_door_w_buf = v.clone();
                if let Ok(val) = v.trim().parse::<f64>()
                    && val > 0.0
                {
                    self.project_snapshot.bible.scale_anchors.door_width_m = val;
                    self.sync_project_and_save();
                }
                Task::none()
            }
            DashboardMsg::AnchorFloorHeightChanged(v) => {
                self.dashboard.anchor_floor_h_buf = v.clone();
                if let Ok(val) = v.trim().parse::<f64>()
                    && val > 0.0
                {
                    self.project_snapshot.bible.scale_anchors.floor_height_m = val;
                    self.sync_project_and_save();
                }
                Task::none()
            }
            DashboardMsg::AnchorRoadLaneChanged(v) => {
                self.dashboard.anchor_road_w_buf = v.clone();
                if let Ok(val) = v.trim().parse::<f64>()
                    && val > 0.0
                {
                    self.project_snapshot.bible.scale_anchors.road_lane_width_m = val;
                    self.sync_project_and_save();
                }
                Task::none()
            }
            DashboardMsg::AddReferenceImage => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                    .pick_file()
                {
                    self.project_snapshot
                        .bible
                        .reference_images
                        .push(path.to_string_lossy().to_string());
                    self.sync_project_and_save();
                }
                Task::none()
            }
            DashboardMsg::RemoveReferenceImage(idx) => {
                if idx < self.project_snapshot.bible.reference_images.len() {
                    self.project_snapshot.bible.reference_images.remove(idx);
                    self.sync_project_and_save();
                }
                Task::none()
            }
            DashboardMsg::ApproveAndLockStage1 => {
                match self.project_snapshot.approve_and_lock_stage1() {
                    Ok(()) => {
                        self.dashboard.error_banner = None;
                        self.dashboard.status_toast =
                            Some("Stage 1 locked! Stage 2 Multi-View unlocked.".to_string());
                        self.sync_project_and_save();
                        if let Some(st) = &self.bridge_state {
                            st.emit(BridgeEvent::StageDecisionChanged {
                                stage: PipelineStage::Stage1Ingestion,
                                approved: true,
                                notes: "Approved & locked by user".to_string(),
                            });
                        }
                    }
                    Err(e) => {
                        self.dashboard.error_banner = Some(e.to_string());
                    }
                }
                Task::none()
            }
            DashboardMsg::UnlockStage(stage) => {
                self.project_snapshot.unlock_stage_by_user(stage);
                self.dashboard.status_toast = Some(format!("Unlocked {}", stage.short_label()));
                self.sync_project_and_save();
                Task::none()
            }

            // Stage 2
            DashboardMsg::AttachAngleImage(angle) => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                    .pick_file()
                {
                    let notes = self
                        .dashboard
                        .angle_note_bufs
                        .get(&angle)
                        .cloned()
                        .unwrap_or_default();
                    if let Err(e) = self.project_snapshot.submit_angle_image(
                        angle,
                        path.to_string_lossy().to_string(),
                        notes,
                    ) {
                        self.dashboard.error_banner = Some(e.to_string());
                    } else {
                        self.sync_project_and_save();
                    }
                }
                Task::none()
            }
            DashboardMsg::AngleNoteChanged(angle, note) => {
                self.dashboard.angle_note_bufs.insert(angle, note.clone());
                if let Some(slot) = self.project_snapshot.multi_view.slot_mut(angle) {
                    slot.notes = note;
                    self.sync_project_and_save();
                }
                Task::none()
            }
            DashboardMsg::ApproveAngle(angle) => {
                if let Err(e) = self.project_snapshot.approve_angle(angle) {
                    self.dashboard.error_banner = Some(e.to_string());
                } else {
                    self.dashboard.error_banner = None;
                    self.dashboard.status_toast = Some(format!("Approved {} view.", angle.label()));
                    self.sync_project_and_save();
                }
                Task::none()
            }
            DashboardMsg::RequestAngleRegen(angle) => {
                let reason = self
                    .dashboard
                    .angle_note_bufs
                    .get(&angle)
                    .cloned()
                    .unwrap_or_else(|| "User requested angle regeneration".to_string());
                self.project_snapshot
                    .request_angle_regeneration(angle, reason.clone());
                self.dashboard.status_toast =
                    Some(format!("Requested regeneration for {}.", angle.label()));
                self.sync_project_and_save();
                if let Some(st) = &self.bridge_state {
                    st.emit(BridgeEvent::StageDecisionChanged {
                        stage: PipelineStage::Stage2MultiView,
                        approved: false,
                        notes: format!("Angle {} regeneration: {reason}", angle.key()),
                    });
                }
                Task::none()
            }

            // Stage 3 Blueprint CAD
            DashboardMsg::Canvas(cmsg) => {
                match cmsg {
                    CanvasMsg::PanBy(delta) => {
                        self.dashboard.canvas_state.pan_x_px += delta.x;
                        self.dashboard.canvas_state.pan_y_px += delta.y;
                    }
                    CanvasMsg::ZoomAt { delta_y, cursor } => {
                        let cs = &mut self.dashboard.canvas_state;
                        let old_z = cs.zoom_px_per_m;
                        let factor = (1.0 + delta_y * 0.12).clamp(0.75, 1.35);
                        let new_z = (old_z * factor).clamp(4.0, 90.0);
                        let wx = (cursor.x - 38.0 - cs.pan_x_px) / old_z;
                        let wy = (cursor.y - 26.0 - cs.pan_y_px) / old_z;
                        cs.zoom_px_per_m = new_z;
                        cs.pan_x_px = cursor.x - 38.0 - wx * new_z;
                        cs.pan_y_px = cursor.y - 26.0 - wy * new_z;
                    }
                    CanvasMsg::SelectEntity(id) => {
                        self.dashboard.canvas_state.selected_entity_id = id;
                        self.dashboard
                            .sync_selected_entity_buffers(&self.project_snapshot);
                    }
                    CanvasMsg::MoveScaleMarker {
                        marker_id,
                        x_m,
                        y_m,
                    } => {
                        if let Some(m) = self
                            .project_snapshot
                            .blueprint
                            .scale_markers
                            .iter_mut()
                            .find(|m| m.id == marker_id)
                        {
                            m.x_m = x_m;
                            m.y_m = y_m;
                            self.sync_project_and_save();
                        }
                    }
                }
                Task::none()
            }
            DashboardMsg::ToggleBlueprintLayer(layer) => {
                let layers = &mut self.dashboard.canvas_state.visible_layers;
                if layers.contains(&layer) {
                    layers.remove(&layer);
                } else {
                    layers.insert(layer);
                }
                Task::none()
            }
            DashboardMsg::CanvasZoomIn => {
                let cs = &mut self.dashboard.canvas_state;
                cs.zoom_px_per_m = (cs.zoom_px_per_m * 1.25).clamp(4.0, 90.0);
                Task::none()
            }
            DashboardMsg::CanvasZoomOut => {
                let cs = &mut self.dashboard.canvas_state;
                cs.zoom_px_per_m = (cs.zoom_px_per_m / 1.25).clamp(4.0, 90.0);
                Task::none()
            }
            DashboardMsg::CanvasFitWorld => {
                let view_w = (self.window_size.width - 540.0).max(400.0);
                let view_h = (self.window_size.height - 220.0).max(320.0);
                self.dashboard.canvas_state.fit_world(
                    self.project_snapshot.blueprint.world_width_m,
                    self.project_snapshot.blueprint.world_depth_m,
                    view_w,
                    view_h,
                );
                Task::none()
            }
            DashboardMsg::SelectedWidthChanged(v) => {
                self.dashboard.selected_w_buf = v.clone();
                if let (Some(id), Ok(val)) = (
                    self.dashboard.canvas_state.selected_entity_id.clone(),
                    v.trim().parse::<f64>(),
                ) && val > 0.0
                {
                    if let Some(fp) = self
                        .project_snapshot
                        .blueprint
                        .footprints
                        .iter_mut()
                        .find(|f| f.id == id)
                    {
                        fp.rect.width_m = val;
                    } else if let Some(door) = self
                        .project_snapshot
                        .blueprint
                        .doors
                        .iter_mut()
                        .find(|d| d.id == id)
                    {
                        door.width_m = val;
                    }
                    self.sync_project_and_save();
                }
                Task::none()
            }
            DashboardMsg::SelectedDepthChanged(v) => {
                self.dashboard.selected_d_buf = v.clone();
                if let (Some(id), Ok(val)) = (
                    self.dashboard.canvas_state.selected_entity_id.clone(),
                    v.trim().parse::<f64>(),
                ) && val > 0.0
                {
                    if let Some(fp) = self
                        .project_snapshot
                        .blueprint
                        .footprints
                        .iter_mut()
                        .find(|f| f.id == id)
                    {
                        fp.rect.depth_m = val;
                    }
                    self.sync_project_and_save();
                }
                Task::none()
            }
            DashboardMsg::SelectedHeightChanged(v) => {
                self.dashboard.selected_h_buf = v.clone();
                if let (Some(id), Ok(val)) = (
                    self.dashboard.canvas_state.selected_entity_id.clone(),
                    v.trim().parse::<f64>(),
                ) && val > 0.0
                {
                    if let Some(fp) = self
                        .project_snapshot
                        .blueprint
                        .footprints
                        .iter_mut()
                        .find(|f| f.id == id)
                    {
                        fp.height_m = val;
                    } else if let Some(door) = self
                        .project_snapshot
                        .blueprint
                        .doors
                        .iter_mut()
                        .find(|d| d.id == id)
                    {
                        door.height_m = val;
                    }
                    self.sync_project_and_save();
                }
                Task::none()
            }
            DashboardMsg::ApproveAndLockStage3 => {
                match self.project_snapshot.approve_and_lock_stage3() {
                    Ok(()) => {
                        self.dashboard.error_banner = None;
                        self.dashboard.status_toast =
                            Some("Stage 3 Master Blueprint locked!".to_string());
                        self.sync_project_and_save();
                    }
                    Err(e) => self.dashboard.error_banner = Some(e.to_string()),
                }
                Task::none()
            }

            // Stage 4 & 6
            DashboardMsg::AttachAssetImage(asset_id) => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                    .pick_file()
                    && let Some(asset) = self
                        .project_snapshot
                        .assets
                        .iter_mut()
                        .find(|a| a.asset_id == asset_id)
                {
                    asset.isolated_image_path = Some(path.to_string_lossy().to_string());
                    self.sync_project_and_save();
                }
                Task::none()
            }
            DashboardMsg::ApproveAndLockStage4 => {
                match self.project_snapshot.approve_and_lock_stage4() {
                    Ok(()) => {
                        self.dashboard.error_banner = None;
                        self.dashboard.status_toast =
                            Some("Stage 4 Asset Catalog locked & validated!".to_string());
                        self.sync_project_and_save();
                    }
                    Err(e) => self.dashboard.error_banner = Some(e.to_string()),
                }
                Task::none()
            }
            DashboardMsg::ExportEngineBundle => {
                if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                    match export_engine_bundle(&self.project_snapshot, &dir) {
                        Ok(manifest) => {
                            self.project_snapshot.telemetry.push_log(
                                "User",
                                "stage6-export",
                                "export_engine_bundle",
                                format!("Exported Stage 6 bundle to {}", manifest.display()),
                                ActivityLevel::Success,
                            );
                            self.dashboard.status_toast =
                                Some(format!("Exported bundle to {}", dir.display()));
                            self.sync_project_and_save();
                        }
                        Err(e) => {
                            self.dashboard.error_banner = Some(format!("Export failed: {e}"));
                        }
                    }
                }
                Task::none()
            }

            // Stage 5
            DashboardMsg::RunScaleValidation => {
                self.project_snapshot.recompute_validation();
                let passed = self.project_snapshot.last_validation.passed;
                let errs = self.project_snapshot.last_validation.error_count;
                let warns = self.project_snapshot.last_validation.warning_count;
                self.project_snapshot.telemetry.push_log(
                    "User",
                    "lutetian-core",
                    "run_scale_validation",
                    format!("Manual validation: passed={passed}, errors={errs}, warnings={warns}"),
                    if passed {
                        ActivityLevel::Success
                    } else {
                        ActivityLevel::Error
                    },
                );
                self.dashboard.status_toast = Some(if passed {
                    "Scale validation passed (0 errors).".to_string()
                } else {
                    format!("Scale validation found {errs} error(s).")
                });
                self.sync_project_and_save();
                Task::none()
            }
            DashboardMsg::InspectComplianceEntity(entity_id) => {
                self.dashboard.active_tab = DashboardTab::Stage3Blueprint;
                self.dashboard.canvas_state.selected_entity_id = Some(entity_id.clone());
                self.dashboard
                    .sync_selected_entity_buffers(&self.project_snapshot);
                if let Some(fp) = self
                    .project_snapshot
                    .blueprint
                    .footprints
                    .iter()
                    .find(|f| f.id == entity_id)
                {
                    let c = fp.rect.center();
                    self.dashboard.canvas_state.focus_world_point(
                        c.x_m,
                        c.y_m,
                        (self.window_size.width - 540.0).max(400.0),
                        (self.window_size.height - 220.0).max(320.0),
                    );
                } else if let Some(door) = self
                    .project_snapshot
                    .blueprint
                    .doors
                    .iter()
                    .find(|d| d.id == entity_id)
                {
                    self.dashboard.canvas_state.focus_world_point(
                        door.position_m.x_m,
                        door.position_m.y_m,
                        (self.window_size.width - 540.0).max(400.0),
                        (self.window_size.height - 220.0).max(320.0),
                    );
                }
                Task::none()
            }

            // Settings
            DashboardMsg::ThemeChanged(mode) => {
                self.config.theme = mode;
                let _ = self.storage.save_config(&self.config);
                Task::none()
            }
            DashboardMsg::BridgePortChanged(v) => {
                self.dashboard.bridge_port_buf = v;
                Task::none()
            }
            DashboardMsg::ApplyBridgePort => {
                if let Ok(port) = self.dashboard.bridge_port_buf.trim().parse::<u16>()
                    && port >= 1024
                {
                    self.config.bridge_port = port;
                    let _ = self.storage.save_config(&self.config);
                    self.dashboard.status_toast = Some(format!(
                        "Saved bridge port {port} (active on next restart; currently :{}).",
                        self.dashboard.bridge_port
                    ));
                } else {
                    self.dashboard.error_banner =
                        Some("Bridge port must be between 1024 and 65535.".to_string());
                }
                Task::none()
            }
            DashboardMsg::CopyMcpConfigJson => {
                let exe_path = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|d| d.join("lutetian-mcp.exe")))
                    .unwrap_or_else(|| PathBuf::from("lutetian-mcp.exe"));
                let snippet = serde_json::json!({
                    "mcpServers": {
                        "lutetian": {
                            "command": exe_path.to_string_lossy(),
                            "args": []
                        }
                    }
                });
                let text = serde_json::to_string_pretty(&snippet).unwrap_or_default();
                self.dashboard.status_toast =
                    Some("Copied Antigravity MCP config to clipboard.".to_string());
                iced::clipboard::write(text)
            }
            DashboardMsg::OpenPluginFolder => {
                let candidate = PathBuf::from("antigravity-plugin");
                let target = if candidate.exists() {
                    candidate
                } else {
                    self.storage.data_dir().to_path_buf()
                };
                let _ = open::that(target);
                Task::none()
            }
            DashboardMsg::ToggleAutoCheckUpdates(v) => {
                self.config.auto_check_updates = v;
                let _ = self.storage.save_config(&self.config);
                Task::none()
            }
            DashboardMsg::ToggleIncludePrereleases(v) => {
                self.config.include_prereleases = v;
                let _ = self.storage.save_config(&self.config);
                Task::none()
            }
            DashboardMsg::CheckUpdatesNow => {
                if self.dashboard.update_checking {
                    return Task::none();
                }
                self.dashboard.update_checking = true;
                self.dashboard.update_error = None;
                let updater = self.updater.clone();
                let include_pre = self.config.include_prereleases;
                Task::perform(
                    async move {
                        updater
                            .check_for_updates_with_channel(env!("CARGO_PKG_VERSION"), include_pre)
                            .await
                            .map_err(|e| e.to_string())
                    },
                    Message::UpdateCheckFinished,
                )
            }
            DashboardMsg::DownloadAndInstallUpdate => {
                if self.dashboard.update_downloading {
                    return Task::none();
                }
                let Some(res) = &self.dashboard.update_status else {
                    return Task::none();
                };
                let Some(asset) = res.installer_asset.clone() else {
                    let _ = open::that(&res.release_url);
                    return Task::none();
                };
                self.dashboard.update_downloading = true;
                self.dashboard.update_download_progress = Some(0);
                let updater = self.updater.clone();
                let (tx, rx) = unbounded_channel::<Message>();
                tokio::spawn(async move {
                    let tx_prog = tx.clone();
                    let res = updater
                        .download_installer_with_progress(&asset, move |p| {
                            let _ = tx_prog.send(Message::UpdateDownloadProgress(p.percent()));
                        })
                        .await
                        .map_err(|e| e.to_string());
                    let _ = tx.send(Message::UpdateDownloadFinished(res));
                });
                Task::stream(tokio_rx_stream(rx))
            }
            DashboardMsg::OpenReleaseUrl(url) => {
                let _ = open::that(url);
                Task::none()
            }
        }
    }

    fn spawn_auto_update_check(&self) -> Task<Message> {
        let updater = self.updater.clone();
        let include_pre = self.config.include_prereleases;
        Task::perform(
            async move {
                if !has_internet_connectivity().await {
                    return None;
                }
                Some(
                    updater
                        .check_for_updates_with_channel(env!("CARGO_PKG_VERSION"), include_pre)
                        .await
                        .map_err(|e| e.to_string()),
                )
            },
            Message::AutoUpdateCheckFinished,
        )
    }
}

fn tokio_rx_stream(
    mut rx: UnboundedReceiver<Message>,
) -> impl iced::futures::Stream<Item = Message> {
    stream::channel(64, async move |mut output| {
        while let Some(msg) = rx.recv().await {
            if output.send(msg).await.is_err() {
                break;
            }
        }
    })
}

fn dashboard_platform_specific() -> window::settings::PlatformSpecific {
    #[cfg(target_os = "windows")]
    {
        window::settings::PlatformSpecific {
            skip_taskbar: false,
            undecorated_shadow: false,
            corner_preference: window::settings::platform::CornerPreference::DoNotRound,
            ..Default::default()
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        window::settings::PlatformSpecific::default()
    }
}

fn open_dashboard_task(icon: Option<window::Icon>) -> (window::Id, Task<Message>) {
    let (id, open) = window::open(window::Settings {
        size: Size::new(1180.0, 760.0),
        min_size: Some(Size::new(780.0, 560.0)),
        position: window::Position::Centered,
        decorations: false,
        transparent: true,
        icon,
        exit_on_close_request: false,
        platform_specific: dashboard_platform_specific(),
        ..Default::default()
    });
    (id, open.map(Message::WindowOpened))
}

fn bridge_stream() -> impl iced::futures::Stream<Item = AppBridgeMsg> {
    stream::channel(64, async move |mut output| {
        let rx = BRIDGE_UI_RX
            .get()
            .and_then(|m| m.lock().ok())
            .and_then(|mut g| g.take());
        if let Some(mut rx) = rx {
            while let Some(ev) = rx.recv().await {
                if output.send(ev).await.is_err() {
                    break;
                }
            }
        } else {
            std::future::pending::<()>().await;
        }
    })
}
