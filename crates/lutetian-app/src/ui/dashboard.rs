use crate::ui::blueprint_canvas::{BlueprintCanvasState, CanvasMsg, CanvasToolMode};
use crate::ui::icons::{UiIcon, app_logo, cached_image_handle, ui_icon};
use crate::ui::mission_control::view_mission_control;
use crate::ui::settings_view::view_settings_tab;
use crate::ui::stage_views::{
    view_assets_tab, view_blueprint_tab, view_multiview_tab, view_stage1_tab,
};
use crate::ui::theme::{
    ThemePalette, aligned_text, badge_style, card_style, font_for_ui, ghost_button_style,
    hairline_divider, nav_button_style, palette_for, pill_button_style, rounded_sidebar_style,
    rounded_titlebar_style, square_icon_button, window_close_button_style,
    wrap_frameless_resize_frame,
};
use iced::font::Weight;
use iced::widget::text::Wrapping;
use iced::widget::{
    Space, Stack, button, column, container, image, mouse_area, progress_bar, row, text, tooltip,
};
use iced::window;
use iced::{Alignment, Background, Border, Color, ContentFit, Element, Font, Length, Padding};
use lutetian_core::{
    AngleKind, AppConfig, BlueprintLayer, ComplianceStatus, LutetianProject, PipelineStage,
    UiThemeMode, UpdateCheckResult,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardTab {
    Stage1Bible,
    Stage2MultiView,
    Stage3Blueprint,
    Stage4Assets,
    MissionControl,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSubTab {
    General,
    Appearance,
    Updates,
}

#[derive(Debug)]
pub struct DashboardState {
    pub active_tab: DashboardTab,
    pub settings_subtab: SettingsSubTab,
    pub is_maximized: bool,
    pub sidebar_collapsed: bool,
    pub titlebar_close_hovered: bool,
    pub canvas_state: BlueprintCanvasState,

    // Lightbox image preview overlay (title, path)
    pub lightbox_image: Option<(String, String)>,
    pub selected_asset_id: Option<String>,

    // Editable numeric string buffers synced with active project
    pub anchor_human_buf: String,
    pub anchor_door_h_buf: String,
    pub anchor_door_w_buf: String,
    pub anchor_floor_h_buf: String,
    pub anchor_road_w_buf: String,
    pub anchor_veh_len_buf: String,
    pub anchor_veh_wid_buf: String,

    pub selected_x_buf: String,
    pub selected_y_buf: String,
    pub selected_w_buf: String,
    pub selected_d_buf: String,
    pub selected_h_buf: String,
    pub selected_rot_buf: String,
    pub selected_elev_buf: String,
    pub selected_floors_buf: String,
    pub selected_buf_id: Option<String>,

    pub angle_note_bufs: BTreeMap<AngleKind, String>,
    pub bridge_port_buf: String,
    pub installed_skills_count: usize,
    pub status_toast: Option<String>,
    pub error_banner: Option<String>,

    // Bridge status
    pub bridge_port: u16,

    // Updates
    pub update_checking: bool,
    pub update_downloading: bool,
    pub update_download_progress: Option<u8>,
    pub update_status: Option<UpdateCheckResult>,
    pub update_error: Option<String>,
}

impl DashboardState {
    pub fn new(
        config: &AppConfig,
        project: &LutetianProject,
        bridge_port: u16,
        installed_skills_count: usize,
    ) -> Self {
        let mut state = Self {
            active_tab: DashboardTab::Stage3Blueprint,
            settings_subtab: SettingsSubTab::General,
            is_maximized: false,
            sidebar_collapsed: false,
            titlebar_close_hovered: false,
            canvas_state: BlueprintCanvasState::default(),
            lightbox_image: None,
            selected_asset_id: None,
            anchor_human_buf: String::new(),
            anchor_door_h_buf: String::new(),
            anchor_door_w_buf: String::new(),
            anchor_floor_h_buf: String::new(),
            anchor_road_w_buf: String::new(),
            anchor_veh_len_buf: String::new(),
            anchor_veh_wid_buf: String::new(),
            selected_x_buf: String::new(),
            selected_y_buf: String::new(),
            selected_w_buf: String::new(),
            selected_d_buf: String::new(),
            selected_h_buf: String::new(),
            selected_rot_buf: String::new(),
            selected_elev_buf: String::new(),
            selected_floors_buf: String::new(),
            selected_buf_id: None,
            angle_note_bufs: BTreeMap::new(),
            bridge_port_buf: config.bridge_port.to_string(),
            installed_skills_count,
            status_toast: None,
            error_banner: None,
            bridge_port,
            update_checking: false,
            update_downloading: false,
            update_download_progress: None,
            update_status: None,
            update_error: None,
        };
        state.sync_from_project(project);
        state.canvas_state.fit_world(
            project.blueprint.world_width_m,
            project.blueprint.world_depth_m,
            740.0,
            480.0,
        );
        state
    }

    pub fn sync_from_project(&mut self, project: &LutetianProject) {
        let a = &project.bible.scale_anchors;
        self.anchor_human_buf = format!("{:.2}", a.human_height_m);
        self.anchor_door_h_buf = format!("{:.2}", a.door_height_m);
        self.anchor_door_w_buf = format!("{:.2}", a.door_width_m);
        self.anchor_floor_h_buf = format!("{:.2}", a.floor_height_m);
        self.anchor_road_w_buf = format!("{:.2}", a.road_lane_width_m);
        self.anchor_veh_len_buf = format!("{:.2}", a.vehicle_length_m);
        self.anchor_veh_wid_buf = format!("{:.2}", a.vehicle_width_m);

        for slot in &project.multi_view.slots {
            self.angle_note_bufs.insert(slot.angle, slot.notes.clone());
        }
        self.sync_selected_entity_buffers(project);
    }

    pub fn sync_selected_entity_buffers(&mut self, project: &LutetianProject) {
        let sel = self.canvas_state.selected_entity_id.as_deref();
        self.selected_buf_id = sel.map(str::to_string);
        if let Some(id) = sel {
            if let Some(fp) = project.blueprint.footprints.iter().find(|f| f.id == id) {
                self.selected_x_buf = format!("{:.2}", fp.rect.x_m);
                self.selected_y_buf = format!("{:.2}", fp.rect.y_m);
                self.selected_w_buf = format!("{:.2}", fp.rect.width_m);
                self.selected_d_buf = format!("{:.2}", fp.rect.depth_m);
                self.selected_h_buf = format!("{:.2}", fp.height_m);
                self.selected_rot_buf = format!("{:.1}", fp.rect.rotation_deg);
                self.selected_elev_buf = format!("{:.2}", fp.base_elevation_m);
                self.selected_floors_buf = fp.floor_count.to_string();
                return;
            }
            if let Some(door) = project.blueprint.doors.iter().find(|d| d.id == id) {
                self.selected_x_buf = format!("{:.2}", door.position_m.x_m);
                self.selected_y_buf = format!("{:.2}", door.position_m.y_m);
                self.selected_w_buf = format!("{:.2}", door.width_m);
                self.selected_d_buf = format!("{:.2}", door.sill_elevation_m);
                self.selected_h_buf = format!("{:.2}", door.height_m);
                self.selected_rot_buf.clear();
                self.selected_elev_buf = format!("{:.2}", door.sill_elevation_m);
                self.selected_floors_buf.clear();
                return;
            }
        }
        self.selected_x_buf.clear();
        self.selected_y_buf.clear();
        self.selected_w_buf.clear();
        self.selected_d_buf.clear();
        self.selected_h_buf.clear();
        self.selected_rot_buf.clear();
        self.selected_elev_buf.clear();
        self.selected_floors_buf.clear();
    }
}

#[derive(Debug, Clone)]
pub enum DashboardMsg {
    // Window chrome
    DragTitlebar,
    MinimizeWindow,
    ToggleMaximize,
    CloseWindow,
    TitlebarCloseHovered(bool),
    ResizeWindow(window::Direction),

    // Navigation & Lightbox
    SwitchTab(DashboardTab),
    ToggleSidebar,
    SwitchSettingsSubTab(SettingsSubTab),
    OpenLightbox { title: String, path: String },
    CloseLightbox,

    // Project lifecycle
    NewEmptyProject,
    LoadDemoProject,
    OpenProjectFile,
    SaveProjectFile,
    DismissToast,

    // Stage 1: Project Bible & Scale Anchors
    BibleTitleChanged(String),
    BibleArtStyleChanged(String),
    BibleMoodChanged(String),
    BibleLightingChanged(String),
    BiblePaletteChanged(String),
    AnchorHumanChanged(String),
    AnchorDoorHeightChanged(String),
    AnchorDoorWidthChanged(String),
    AnchorFloorHeightChanged(String),
    AnchorRoadLaneChanged(String),
    AnchorVehicleLenChanged(String),
    AnchorVehicleWidthChanged(String),
    AddReferenceImage,
    RemoveReferenceImage(usize),
    ApproveAndLockStage1,
    UnlockStage(PipelineStage),

    // Stage 2: Multi-View
    AttachAngleImage(AngleKind),
    AngleNoteChanged(AngleKind, String),
    ApproveAngle(AngleKind),
    RequestAngleRegen(AngleKind),

    // Stage 3: Blueprint CAD
    Canvas(CanvasMsg),
    SetCanvasTool(CanvasToolMode),
    ClearMeasureRuler,
    ToggleBlueprintLayer(BlueprintLayer),
    CanvasZoomIn,
    CanvasZoomOut,
    CanvasFitWorld,
    SelectedPosXChanged(String),
    SelectedPosYChanged(String),
    SelectedWidthChanged(String),
    SelectedDepthChanged(String),
    SelectedHeightChanged(String),
    SelectedRotChanged(String),
    SelectedElevChanged(String),
    SelectedFloorsChanged(String),
    AddBlueprintFootprint,
    AddBlueprintDoor,
    DeleteSelectedEntity,
    ApproveAndLockStage3,

    // Stage 4 & 6: Assets & Export
    AttachAssetImage(String),
    JumpToAsset(String),
    SyncAssetBbox(String),
    ApproveAndLockStage4,
    ExportEngineBundle,

    // Stage 5: Mission Control & Validation
    RunScaleValidation,
    InspectComplianceEntity(String),

    // Settings
    ThemeChanged(UiThemeMode),
    BridgePortChanged(String),
    ApplyBridgePort,
    CopyMcpConfigJson,
    InstallAntigravityPlugin,
    OpenPluginFolder,
    ToggleAutoCheckUpdates(bool),
    ToggleIncludePrereleases(bool),
    CheckUpdatesNow,
    DownloadAndInstallUpdate,
    OpenReleaseUrl(String),
}

pub fn view_dashboard<'a>(
    state: &'a DashboardState,
    config: &'a AppConfig,
    project: &'a LutetianProject,
) -> Element<'a, DashboardMsg> {
    let mode = config.theme;
    let p = palette_for(mode);
    let ui_font = font_for_ui(Weight::Normal);
    let ui_font_semi = font_for_ui(Weight::Medium);
    let ui_font_bold = font_for_ui(Weight::Bold);

    let titlebar =
        view_custom_titlebar(state, project, mode, p, ui_font, ui_font_semi, ui_font_bold);
    let sidebar = view_sidebar(state, project, mode, p, ui_font, ui_font_semi);

    let main_view: Element<'a, DashboardMsg> = match state.active_tab {
        DashboardTab::Stage1Bible => {
            view_stage1_tab(state, project, mode, p, ui_font, ui_font_semi, ui_font_bold)
        }
        DashboardTab::Stage2MultiView => {
            view_multiview_tab(state, project, mode, p, ui_font, ui_font_semi, ui_font_bold)
        }
        DashboardTab::Stage3Blueprint => {
            view_blueprint_tab(state, project, mode, p, ui_font, ui_font_semi, ui_font_bold)
        }
        DashboardTab::Stage4Assets => {
            view_assets_tab(state, project, mode, p, ui_font, ui_font_semi, ui_font_bold)
        }
        DashboardTab::MissionControl => view_mission_control(
            project,
            state.bridge_port,
            mode,
            p,
            ui_font,
            ui_font_semi,
            ui_font_bold,
        ),
        DashboardTab::Settings => {
            view_settings_tab(state, config, mode, p, ui_font, ui_font_semi, ui_font_bold)
        }
    };

    let live_strip = view_live_mission_strip(state, project, mode, p, ui_font, ui_font_semi);

    let v_divider =
        container(Space::new().width(1).height(Length::Fill)).style(move |_| container::Style {
            background: Some(Background::Color(p.border_subtle)),
            ..Default::default()
        });

    let center_col = column![
        container(main_view)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::from([20, 24])),
        hairline_divider(mode),
        live_strip,
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    let workspace = row![sidebar, v_divider, center_col]
        .width(Length::Fill)
        .height(Length::Fill);

    let shell = column![titlebar, hairline_divider(mode), workspace]
        .width(Length::Fill)
        .height(Length::Fill);

    let base_shell = wrap_frameless_resize_frame(
        shell.into(),
        mode,
        12.0,
        state.is_maximized,
        DashboardMsg::ResizeWindow,
    );

    if let Some((title, path)) = &state.lightbox_image {
        let modal =
            view_lightbox_overlay(title, path, mode, p, ui_font, ui_font_semi, ui_font_bold);
        Stack::with_children(vec![base_shell, modal])
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        base_shell
    }
}

fn view_lightbox_overlay<'a>(
    title: &'a str,
    path: &'a str,
    mode: UiThemeMode,
    p: ThemePalette,
    ui_font: Font,
    ui_font_semi: Font,
    ui_font_bold: Font,
) -> Element<'a, DashboardMsg> {
    let img_body: Element<'a, DashboardMsg> = if let Some(handle) = cached_image_handle(path) {
        image(handle)
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(ContentFit::Contain)
            .into()
    } else {
        container(
            text(format!("Image file not found on disk: {path}"))
                .size(13)
                .font(Font::MONOSPACE)
                .color(p.text_secondary),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    };

    let card = container(
        column![
            row![
                ui_icon(UiIcon::PhotoLibrary, 16.0, p.accent_text),
                text(title)
                    .size(14.5)
                    .font(ui_font_bold)
                    .color(p.text_primary),
                Space::new().width(Length::Fill),
                text(path)
                    .size(11.5)
                    .font(Font::MONOSPACE)
                    .color(p.text_secondary),
                button(
                    row![
                        ui_icon(UiIcon::Close, 14.0, p.text_primary),
                        text("Close").size(12).font(ui_font_semi),
                    ]
                    .spacing(5)
                    .align_y(Alignment::Center),
                )
                .on_press(DashboardMsg::CloseLightbox)
                .padding([6, 11])
                .style(pill_button_style(mode)),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            hairline_divider(mode),
            container(img_body)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(8),
            text("Press Esc or Close to return")
                .size(11)
                .font(ui_font)
                .color(p.text_muted),
        ]
        .spacing(10),
    )
    .padding(18)
    .width(Length::Fixed(760.0))
    .height(Length::Fixed(520.0))
    .style(card_style(mode, 12.0));

    mouse_area(
        container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(move |_| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.03, 0.02, 0.05, 0.78))),
                ..Default::default()
            }),
    )
    .on_press(DashboardMsg::CloseLightbox)
    .into()
}

fn view_custom_titlebar<'a>(
    state: &'a DashboardState,
    project: &'a LutetianProject,
    mode: UiThemeMode,
    p: ThemePalette,
    ui_font: Font,
    ui_font_semi: Font,
    ui_font_bold: Font,
) -> Element<'a, DashboardMsg> {
    let connected = project.telemetry.heartbeat.connected;
    let bridge_badge = container(
        row![
            ui_icon(
                UiIcon::Hub,
                13.0,
                if connected { p.success } else { p.accent_text }
            ),
            text(if connected {
                format!("MCP Live · :{}", state.bridge_port)
            } else {
                format!("Bridge :{}", state.bridge_port)
            })
            .size(11)
            .font(ui_font_semi)
            .color(if connected { p.success } else { p.accent_text })
            .wrapping(Wrapping::None),
        ]
        .spacing(5)
        .align_y(Alignment::Center),
    )
    .padding([3, 9])
    .style(badge_style(
        if connected {
            p.success_soft
        } else {
            p.accent_soft
        },
        if connected { p.success } else { p.accent_text },
        p.border_subtle,
    ));

    let brand_strip = mouse_area(
        row![
            app_logo(22.0),
            text("Lutetian")
                .size(13)
                .font(ui_font_bold)
                .color(p.text_primary),
            text("·").size(12).color(p.text_muted),
            text(&project.name)
                .size(12)
                .font(ui_font)
                .color(p.text_secondary)
                .wrapping(Wrapping::None),
            bridge_badge,
            Space::new().width(Length::Fill),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fixed(36.0)),
    )
    .on_press(DashboardMsg::DragTitlebar)
    .on_double_click(DashboardMsg::ToggleMaximize);

    let project_actions = row![
        titlebar_action_btn(
            "Sample",
            UiIcon::RestartAlt,
            DashboardMsg::LoadDemoProject,
            mode,
            p,
            ui_font_semi
        ),
        titlebar_action_btn(
            "Open",
            UiIcon::FolderOpen,
            DashboardMsg::OpenProjectFile,
            mode,
            p,
            ui_font_semi
        ),
        titlebar_action_btn(
            "Save",
            UiIcon::Save,
            DashboardMsg::SaveProjectFile,
            mode,
            p,
            ui_font_semi
        ),
    ]
    .spacing(4)
    .align_y(Alignment::Center);

    let sidebar_icon = if state.sidebar_collapsed {
        UiIcon::SidebarClosed
    } else {
        UiIcon::SidebarOpen
    };
    let sidebar_btn = tooltip(
        square_icon_button(
            ui_icon(sidebar_icon, 15.0, p.text_secondary),
            28.0,
            DashboardMsg::ToggleSidebar,
            ghost_button_style(mode, false),
        ),
        text("Toggle Sidebar")
            .size(12)
            .font(ui_font)
            .color(p.text_primary),
        tooltip::Position::Bottom,
    )
    .gap(6)
    .padding(6)
    .style(sidebar_tooltip_style(mode));

    let min_btn = square_icon_button(
        ui_icon(UiIcon::Remove, 15.0, p.text_secondary),
        28.0,
        DashboardMsg::MinimizeWindow,
        ghost_button_style(mode, false),
    );
    let max_icon = if state.is_maximized {
        UiIcon::FilterNone
    } else {
        UiIcon::CropSquare
    };
    let max_btn = square_icon_button(
        ui_icon(max_icon, 14.0, p.text_secondary),
        28.0,
        DashboardMsg::ToggleMaximize,
        ghost_button_style(mode, false),
    );
    let close_btn = mouse_area(square_icon_button(
        ui_icon(
            UiIcon::Close,
            15.0,
            if state.titlebar_close_hovered {
                Color::WHITE
            } else {
                p.text_secondary
            },
        ),
        28.0,
        DashboardMsg::CloseWindow,
        window_close_button_style(mode),
    ))
    .on_enter(DashboardMsg::TitlebarCloseHovered(true))
    .on_exit(DashboardMsg::TitlebarCloseHovered(false));

    let controls = row![
        project_actions,
        Space::new().width(6),
        sidebar_btn,
        min_btn,
        max_btn,
        close_btn
    ]
    .spacing(4)
    .align_y(Alignment::Center);

    let top_radius = if state.is_maximized { 0.0 } else { 11.0 };

    container(
        row![brand_strip, controls]
            .spacing(10)
            .align_y(Alignment::Center)
            .width(Length::Fill),
    )
    .padding(Padding::from([2, 12]))
    .width(Length::Fill)
    .height(Length::Fixed(38.0))
    .style(rounded_titlebar_style(mode, top_radius))
    .into()
}

fn titlebar_action_btn<'a>(
    label: &'a str,
    icon: UiIcon,
    msg: DashboardMsg,
    mode: UiThemeMode,
    p: ThemePalette,
    ui_font_semi: Font,
) -> Element<'a, DashboardMsg> {
    button(
        row![
            ui_icon(icon, 13.0, p.text_secondary),
            text(label)
                .size(11.5)
                .font(ui_font_semi)
                .color(p.text_secondary),
        ]
        .spacing(5)
        .align_y(Alignment::Center),
    )
    .on_press(msg)
    .padding([4, 8])
    .style(ghost_button_style(mode, false))
    .into()
}

fn sidebar_tooltip_style(mode: UiThemeMode) -> impl Fn(&iced::Theme) -> container::Style {
    move |_theme| {
        let p = palette_for(mode);
        container::Style {
            background: Some(Background::Color(p.bg_elevated)),
            border: Border {
                color: p.border_subtle,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }
    }
}

fn view_sidebar<'a>(
    state: &'a DashboardState,
    project: &'a LutetianProject,
    mode: UiThemeMode,
    p: ThemePalette,
    ui_font: Font,
    ui_font_semi: Font,
) -> Element<'a, DashboardMsg> {
    let collapsed = state.sidebar_collapsed;

    let nav_items = [
        (
            DashboardTab::Stage1Bible,
            UiIcon::MenuBook,
            "1 · Project Bible".to_string(),
            Some(project.is_stage_locked(PipelineStage::Stage1Ingestion)),
        ),
        (
            DashboardTab::Stage2MultiView,
            UiIcon::PhotoLibrary,
            "2 · Multi-View".to_string(),
            Some(project.is_stage_locked(PipelineStage::Stage2MultiView)),
        ),
        (
            DashboardTab::Stage3Blueprint,
            UiIcon::Layers,
            "3 · Blueprint CAD".to_string(),
            Some(project.is_stage_locked(PipelineStage::Stage3Blueprint)),
        ),
        (
            DashboardTab::Stage4Assets,
            UiIcon::ViewInAr,
            "4 · Asset Catalog".to_string(),
            Some(project.is_stage_locked(PipelineStage::Stage4Assets)),
        ),
        (
            DashboardTab::MissionControl,
            UiIcon::Monitoring,
            "5 · Mission Control".to_string(),
            None,
        ),
    ];

    let mut nav_col = column![].spacing(5);
    for (tab, icon, label, locked_opt) in nav_items {
        nav_col = nav_col.push(sidebar_nav_button(
            tab,
            icon,
            label,
            locked_opt,
            state.active_tab,
            collapsed,
            mode,
            p,
            ui_font,
            ui_font_semi,
        ));
    }

    let settings_btn = sidebar_nav_button(
        DashboardTab::Settings,
        UiIcon::Settings,
        "Settings".to_string(),
        None,
        state.active_tab,
        collapsed,
        mode,
        p,
        ui_font,
        ui_font_semi,
    );

    let bottom_radius = if state.is_maximized { 0.0 } else { 11.0 };
    let sidebar_width = if collapsed { 64.0 } else { 220.0 };

    let mut sidebar_items: Vec<Element<'a, DashboardMsg>> =
        vec![nav_col.into(), Space::new().height(Length::Fill).into()];

    if let Some(update_banner) =
        sidebar_update_banner(state, collapsed, mode, p, ui_font, ui_font_semi)
    {
        sidebar_items.push(update_banner);
    }

    sidebar_items.push(hairline_divider(mode));
    sidebar_items.push(settings_btn);

    container(column(sidebar_items).spacing(8).height(Length::Fill))
        .width(Length::Fixed(sidebar_width))
        .height(Length::Fill)
        .padding(if collapsed {
            Padding::from([16, 10])
        } else {
            Padding::from([16, 14])
        })
        .style(rounded_sidebar_style(mode, bottom_radius))
        .into()
}

#[allow(clippy::too_many_arguments)]
fn sidebar_nav_button<'a>(
    tab: DashboardTab,
    icon: UiIcon,
    label: String,
    locked_opt: Option<bool>,
    active_tab: DashboardTab,
    collapsed: bool,
    mode: UiThemeMode,
    p: ThemePalette,
    ui_font: Font,
    ui_font_semi: Font,
) -> Element<'a, DashboardMsg> {
    let is_active = active_tab == tab;
    let lbl_font = if is_active { ui_font_semi } else { ui_font };
    let icon_color = if is_active {
        p.accent_text
    } else {
        p.text_secondary
    };

    let content: Element<'a, DashboardMsg> = if collapsed {
        container(ui_icon(icon, 18.0, icon_color))
            .width(Length::Fill)
            .center_x(Length::Fill)
            .into()
    } else {
        let mut items: Vec<Element<'a, DashboardMsg>> = vec![
            ui_icon(icon, 17.0, icon_color).into(),
            aligned_text(label.clone(), 13.0, lbl_font, None),
        ];
        if let Some(is_locked) = locked_opt {
            items.push(
                ui_icon(
                    if is_locked {
                        UiIcon::Lock
                    } else {
                        UiIcon::LockOpen
                    },
                    13.0,
                    if is_locked { p.success } else { p.text_muted },
                )
                .into(),
            );
        }
        row(items)
            .spacing(10)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .into()
    };

    let btn = button(content)
        .on_press(DashboardMsg::SwitchTab(tab))
        .padding(if collapsed { [10, 0] } else { [9, 12] })
        .width(Length::Fill)
        .style(nav_button_style(mode, is_active));

    if collapsed {
        tooltip(
            btn,
            text(label).size(12).font(ui_font).color(p.text_primary),
            tooltip::Position::Right,
        )
        .gap(8)
        .padding(6)
        .style(sidebar_tooltip_style(mode))
        .into()
    } else {
        btn.into()
    }
}

fn sidebar_update_banner<'a>(
    state: &'a DashboardState,
    collapsed: bool,
    mode: UiThemeMode,
    p: ThemePalette,
    ui_font: Font,
    ui_font_semi: Font,
) -> Option<Element<'a, DashboardMsg>> {
    let has_update = state.update_status.as_ref().is_some_and(|r| r.has_update);
    if !state.update_downloading && !has_update {
        return None;
    }

    if state.update_downloading {
        let pct = state.update_download_progress.unwrap_or(0);
        let label = format!("Downloading {pct}%");
        if collapsed {
            return Some(
                container(
                    column![
                        ui_icon(UiIcon::Download, 15.0, p.accent_text),
                        progress_bar(0.0..=100.0, pct as f32).girth(3),
                    ]
                    .spacing(4)
                    .align_x(Alignment::Center),
                )
                .padding([6, 6])
                .width(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(Background::Color(p.accent_soft)),
                    border: Border {
                        color: p.border_subtle,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                })
                .into(),
            );
        }
        return Some(
            container(
                column![
                    row![
                        ui_icon(UiIcon::Download, 14.0, p.accent_text),
                        text(label)
                            .size(11.5)
                            .font(ui_font_semi)
                            .color(p.accent_text),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                    progress_bar(0.0..=100.0, pct as f32).girth(4),
                ]
                .spacing(6),
            )
            .padding([8, 10])
            .width(Length::Fill)
            .style(move |_| container::Style {
                background: Some(Background::Color(p.accent_soft)),
                border: Border {
                    color: p.border_subtle,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            })
            .into(),
        );
    }

    let version = state
        .update_status
        .as_ref()
        .map(|r| r.latest_version.as_str())
        .unwrap_or("");
    let full_label = format!("Update · v{version}");
    if collapsed {
        return Some(
            tooltip(
                button(
                    container(ui_icon(UiIcon::Download, 16.0, p.accent_text))
                        .width(Length::Fill)
                        .center_x(Length::Fill),
                )
                .on_press(DashboardMsg::DownloadAndInstallUpdate)
                .padding([8, 8])
                .width(Length::Fill)
                .style(pill_button_style(mode)),
                text(full_label)
                    .size(12)
                    .font(ui_font)
                    .color(p.text_primary),
                tooltip::Position::Right,
            )
            .gap(8)
            .padding(6)
            .style(sidebar_tooltip_style(mode))
            .into(),
        );
    }
    Some(
        button(
            row![
                ui_icon(UiIcon::Download, 14.0, p.accent_text),
                text(full_label)
                    .size(11.5)
                    .font(ui_font_semi)
                    .color(p.accent_text),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .on_press(DashboardMsg::DownloadAndInstallUpdate)
        .padding([8, 10])
        .width(Length::Fill)
        .style(pill_button_style(mode))
        .into(),
    )
}

fn view_live_mission_strip<'a>(
    state: &'a DashboardState,
    project: &'a LutetianProject,
    mode: UiThemeMode,
    p: ThemePalette,
    ui_font: Font,
    ui_font_semi: Font,
) -> Element<'a, DashboardMsg> {
    let hb = &project.telemetry.heartbeat;
    let compliant_cnt = project
        .last_validation
        .compliance_matrix
        .iter()
        .filter(|r| r.status == ComplianceStatus::Compliant)
        .count();
    let pending_cnt = project
        .last_validation
        .compliance_matrix
        .iter()
        .filter(|r| r.status == ComplianceStatus::Pending)
        .count();
    let violated_cnt = project
        .last_validation
        .compliance_matrix
        .iter()
        .filter(|r| r.status == ComplianceStatus::Violated)
        .count();

    let status_dot_col = if violated_cnt > 0 {
        p.danger
    } else if hb.connected {
        p.success
    } else {
        p.accent_text
    };

    let mut strip_items: Vec<Element<'a, DashboardMsg>> = vec![
        ui_icon(UiIcon::Terminal, 14.0, status_dot_col).into(),
        text(format!("[{}] {}", hb.active_skill, hb.current_action))
            .size(11.5)
            .font(ui_font)
            .color(p.text_secondary)
            .wrapping(Wrapping::None)
            .into(),
        Space::new().width(Length::Fill).into(),
    ];

    if let Some(toast) = state
        .error_banner
        .as_deref()
        .or(state.status_toast.as_deref())
    {
        let is_err = state.error_banner.is_some();
        strip_items.push(
            button(text(toast).size(11.0).font(ui_font_semi).color(if is_err {
                p.danger
            } else {
                p.accent_text
            }))
            .on_press(DashboardMsg::DismissToast)
            .padding([3, 8])
            .style(ghost_button_style(mode, false))
            .into(),
        );
    }

    strip_items.push(
        container(
            text(format!("{compliant_cnt} Compliant"))
                .size(11)
                .font(ui_font_semi)
                .color(p.success),
        )
        .padding([3, 8])
        .style(badge_style(p.success_soft, p.success, p.border_subtle))
        .into(),
    );
    strip_items.push(
        container(
            text(format!("{pending_cnt} Pending"))
                .size(11)
                .font(ui_font_semi)
                .color(p.warning),
        )
        .padding([3, 8])
        .style(badge_style(p.warning_soft, p.warning, p.border_subtle))
        .into(),
    );
    strip_items.push(
        container(
            text(format!("{violated_cnt} Violated"))
                .size(11)
                .font(ui_font_semi)
                .color(if violated_cnt > 0 {
                    p.danger
                } else {
                    p.text_muted
                }),
        )
        .padding([3, 8])
        .style(badge_style(
            if violated_cnt > 0 {
                p.danger_soft
            } else {
                p.bg_elevated
            },
            if violated_cnt > 0 {
                p.danger
            } else {
                p.text_muted
            },
            p.border_subtle,
        ))
        .into(),
    );
    strip_items.push(
        button(
            row![
                ui_icon(UiIcon::Monitoring, 13.0, p.text_primary),
                text("Mission Control")
                    .size(11.5)
                    .font(ui_font_semi)
                    .color(p.text_primary),
            ]
            .spacing(5)
            .align_y(Alignment::Center),
        )
        .on_press(DashboardMsg::SwitchTab(DashboardTab::MissionControl))
        .padding([4, 10])
        .style(pill_button_style(mode))
        .into(),
    );

    container(
        row(strip_items)
            .spacing(8)
            .align_y(Alignment::Center)
            .width(Length::Fill),
    )
    .padding(Padding::from([7, 16]))
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(Background::Color(p.bg_sidebar)),
        ..Default::default()
    })
    .into()
}

pub fn page_header<'a, M: 'a>(
    title: impl Into<String>,
    subtitle: Option<String>,
    right_action: Option<Element<'a, M>>,
    p: ThemePalette,
    ui_font: Font,
    ui_font_bold: Font,
) -> Element<'a, M> {
    let mut title_col = column![aligned_text(
        title.into(),
        20.0,
        ui_font_bold,
        Some(p.text_primary),
    )]
    .spacing(4)
    .width(Length::Fill);

    if let Some(sub) = subtitle {
        title_col = title_col.push(aligned_text(sub, 12.5, ui_font, Some(p.text_secondary)));
    }

    let mut items: Vec<Element<'a, M>> = vec![title_col.into()];
    if let Some(act) = right_action {
        items.push(act);
    }

    row(items)
        .spacing(16)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .into()
}
