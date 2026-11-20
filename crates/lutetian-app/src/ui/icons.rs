use iced::widget::image::{Handle as ImageHandle, Image};
use iced::widget::svg::{self, Handle, Svg};
use iced::{Color, ContentFit, Length};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

pub const APP_LOGO_PNG: &[u8] = include_bytes!("../../../../assets/icons/lutetian.png");

// Embedded sample previews for Courtyard Market District demo project
const SAMPLE_CONCEPT_MAIN: &[u8] =
    include_bytes!("../../../../assets/samples/courtyard_concept_main.png");
const SAMPLE_FRONT: &[u8] = include_bytes!("../../../../assets/samples/courtyard_front.png");
const SAMPLE_BACK: &[u8] = include_bytes!("../../../../assets/samples/courtyard_back.png");
const SAMPLE_LEFT: &[u8] = include_bytes!("../../../../assets/samples/courtyard_left.png");
const SAMPLE_RIGHT: &[u8] = include_bytes!("../../../../assets/samples/courtyard_right.png");
const SAMPLE_BIRD_EYE: &[u8] = include_bytes!("../../../../assets/samples/courtyard_bird_eye.png");
const SAMPLE_BLDG_HOUSE: &[u8] = include_bytes!("../../../../assets/samples/bldg_timber_house.png");
const SAMPLE_BLDG_BAKERY: &[u8] = include_bytes!("../../../../assets/samples/bldg_bakery_shop.png");
const SAMPLE_BLDG_TOWER: &[u8] = include_bytes!("../../../../assets/samples/bldg_watchtower.png");
const SAMPLE_PROP_FOUNTAIN: &[u8] =
    include_bytes!("../../../../assets/samples/prop_stone_fountain.png");
const SAMPLE_TREE_PINE: &[u8] = include_bytes!("../../../../assets/samples/tree_pine_01.png");
const SAMPLE_BENCH_WOOD: &[u8] = include_bytes!("../../../../assets/samples/bench_wood_01.png");

static APP_LOGO_HANDLE: OnceLock<ImageHandle> = OnceLock::new();
static IMAGE_FILE_CACHE: OnceLock<Mutex<HashMap<String, Option<ImageHandle>>>> = OnceLock::new();

fn embedded_sample_bytes(key: &str) -> Option<&'static [u8]> {
    match key.replace('\\', "/").as_str() {
        "samples/courtyard_concept_main.png" => Some(SAMPLE_CONCEPT_MAIN),
        "samples/courtyard_front.png" => Some(SAMPLE_FRONT),
        "samples/courtyard_back.png" => Some(SAMPLE_BACK),
        "samples/courtyard_left.png" => Some(SAMPLE_LEFT),
        "samples/courtyard_right.png" => Some(SAMPLE_RIGHT),
        "samples/courtyard_bird_eye.png" => Some(SAMPLE_BIRD_EYE),
        "samples/bldg_timber_house.png" => Some(SAMPLE_BLDG_HOUSE),
        "samples/bldg_bakery_shop.png" => Some(SAMPLE_BLDG_BAKERY),
        "samples/bldg_watchtower.png" => Some(SAMPLE_BLDG_TOWER),
        "samples/prop_stone_fountain.png" => Some(SAMPLE_PROP_FOUNTAIN),
        "samples/tree_pine_01.png" => Some(SAMPLE_TREE_PINE),
        "samples/bench_wood_01.png" => Some(SAMPLE_BENCH_WOOD),
        _ => None,
    }
}

fn app_logo_handle() -> ImageHandle {
    APP_LOGO_HANDLE
        .get_or_init(|| match image::load_from_memory(APP_LOGO_PNG) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let (width, height) = rgba.dimensions();
                ImageHandle::from_rgba(width, height, rgba.into_raw())
            }
            Err(_) => ImageHandle::from_bytes(APP_LOGO_PNG),
        })
        .clone()
}

pub fn load_app_icon_rgba() -> Option<(Vec<u8>, u32, u32)> {
    let img = image::load_from_memory(APP_LOGO_PNG).ok()?.into_rgba8();
    let (w, h) = img.dimensions();
    Some((img.into_raw(), w, h))
}

pub fn app_logo(size: f32) -> Image<ImageHandle> {
    Image::new(app_logo_handle())
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .content_fit(ContentFit::Contain)
}

/// Loads and caches an external PNG/JPG/WebP image handle from disk or embedded sample previews.
pub fn cached_image_handle(path: &str) -> Option<ImageHandle> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }
    let cache = IMAGE_FILE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(guard) = cache.lock()
        && let Some(cached) = guard.get(trimmed)
    {
        return cached.clone();
    }

    let loaded = if Path::new(trimmed).exists() {
        image::open(trimmed).ok().map(|img| {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            ImageHandle::from_rgba(w, h, rgba.into_raw())
        })
    } else if let Some(bytes) = embedded_sample_bytes(trimmed) {
        image::load_from_memory(bytes).ok().map(|img| {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            ImageHandle::from_rgba(w, h, rgba.into_raw())
        })
    } else {
        None
    };

    if let Ok(mut guard) = cache.lock() {
        guard.insert(trimmed.to_string(), loaded.clone());
    }
    loaded
}

// Embedded Material Symbols Rounded UI SVGs
const ICON_ACCESSIBILITY_NEW: &[u8] =
    include_bytes!("../../../../assets/icons/ui/accessibility_new.svg");
const ICON_ADD: &[u8] = include_bytes!("../../../../assets/icons/ui/add.svg");
const ICON_ARCHITECTURE: &[u8] = include_bytes!("../../../../assets/icons/ui/architecture.svg");
const ICON_BOLT: &[u8] = include_bytes!("../../../../assets/icons/ui/bolt.svg");
const ICON_CHECK_CIRCLE: &[u8] = include_bytes!("../../../../assets/icons/ui/check_circle.svg");
const ICON_CLOSE: &[u8] = include_bytes!("../../../../assets/icons/ui/close.svg");
const ICON_COPY: &[u8] = include_bytes!("../../../../assets/icons/ui/content_copy.svg");
const ICON_CROP_SQUARE: &[u8] = include_bytes!("../../../../assets/icons/ui/crop_square.svg");
const ICON_DARK_MODE: &[u8] = include_bytes!("../../../../assets/icons/ui/dark_mode.svg");
const ICON_DELETE: &[u8] = include_bytes!("../../../../assets/icons/ui/delete.svg");
const ICON_DIRECTIONS_CAR: &[u8] = include_bytes!("../../../../assets/icons/ui/directions_car.svg");
const ICON_DOWNLOAD: &[u8] = include_bytes!("../../../../assets/icons/ui/download.svg");
const ICON_FILTER_NONE: &[u8] = include_bytes!("../../../../assets/icons/ui/filter_none.svg");
const ICON_FIT_SCREEN: &[u8] = include_bytes!("../../../../assets/icons/ui/fit_screen.svg");
const ICON_FOLDER_OPEN: &[u8] = include_bytes!("../../../../assets/icons/ui/folder_open.svg");
const ICON_HUB: &[u8] = include_bytes!("../../../../assets/icons/ui/hub.svg");
const ICON_LAYERS: &[u8] = include_bytes!("../../../../assets/icons/ui/layers.svg");
const ICON_LIGHT_MODE: &[u8] = include_bytes!("../../../../assets/icons/ui/light_mode.svg");
const ICON_LOCK: &[u8] = include_bytes!("../../../../assets/icons/ui/lock.svg");
const ICON_LOCK_OPEN: &[u8] = include_bytes!("../../../../assets/icons/ui/lock_open.svg");
const ICON_MENU_BOOK: &[u8] = include_bytes!("../../../../assets/icons/ui/menu_book.svg");
const ICON_MONITORING: &[u8] = include_bytes!("../../../../assets/icons/ui/monitoring.svg");
const ICON_PHOTO_LIBRARY: &[u8] = include_bytes!("../../../../assets/icons/ui/photo_library.svg");
const ICON_REFRESH: &[u8] = include_bytes!("../../../../assets/icons/ui/refresh.svg");
const ICON_REMOVE: &[u8] = include_bytes!("../../../../assets/icons/ui/remove.svg");
const ICON_RESTART_ALT: &[u8] = include_bytes!("../../../../assets/icons/ui/restart_alt.svg");
const ICON_SAVE: &[u8] = include_bytes!("../../../../assets/icons/ui/save.svg");
const ICON_SCHEDULE: &[u8] = include_bytes!("../../../../assets/icons/ui/schedule.svg");
const ICON_SETTINGS: &[u8] = include_bytes!("../../../../assets/icons/ui/settings.svg");
const ICON_SIDEBAR_CLOSED: &[u8] = include_bytes!("../../../../assets/icons/ui/sidebar_closed.svg");
const ICON_SIDEBAR_OPEN: &[u8] = include_bytes!("../../../../assets/icons/ui/sidebar_open.svg");
const ICON_STRAIGHTEN: &[u8] = include_bytes!("../../../../assets/icons/ui/straighten.svg");
const ICON_TERMINAL: &[u8] = include_bytes!("../../../../assets/icons/ui/terminal.svg");
const ICON_VERIFIED_USER: &[u8] = include_bytes!("../../../../assets/icons/ui/verified_user.svg");
const ICON_VIEW_IN_AR: &[u8] = include_bytes!("../../../../assets/icons/ui/view_in_ar.svg");
const ICON_VISIBILITY: &[u8] = include_bytes!("../../../../assets/icons/ui/visibility.svg");
const ICON_WARNING: &[u8] = include_bytes!("../../../../assets/icons/ui/warning.svg");
const ICON_ZOOM_IN: &[u8] = include_bytes!("../../../../assets/icons/ui/zoom_in.svg");
const ICON_ZOOM_OUT: &[u8] = include_bytes!("../../../../assets/icons/ui/zoom_out.svg");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiIcon {
    AccessibilityNew,
    Add,
    Architecture,
    Bolt,
    CheckCircle,
    Close,
    Copy,
    CropSquare,
    DarkMode,
    Delete,
    DirectionsCar,
    Download,
    FilterNone,
    FitScreen,
    FolderOpen,
    Hub,
    Layers,
    LightMode,
    Lock,
    LockOpen,
    MenuBook,
    Monitoring,
    PhotoLibrary,
    Refresh,
    Remove,
    RestartAlt,
    Save,
    Schedule,
    Settings,
    SidebarClosed,
    SidebarOpen,
    Straighten,
    Terminal,
    VerifiedUser,
    ViewInAr,
    Visibility,
    Warning,
    ZoomIn,
    ZoomOut,
}

impl UiIcon {
    pub fn bytes(self) -> &'static [u8] {
        match self {
            Self::AccessibilityNew => ICON_ACCESSIBILITY_NEW,
            Self::Add => ICON_ADD,
            Self::Architecture => ICON_ARCHITECTURE,
            Self::Bolt => ICON_BOLT,
            Self::CheckCircle => ICON_CHECK_CIRCLE,
            Self::Close => ICON_CLOSE,
            Self::Copy => ICON_COPY,
            Self::CropSquare => ICON_CROP_SQUARE,
            Self::DarkMode => ICON_DARK_MODE,
            Self::Delete => ICON_DELETE,
            Self::DirectionsCar => ICON_DIRECTIONS_CAR,
            Self::Download => ICON_DOWNLOAD,
            Self::FilterNone => ICON_FILTER_NONE,
            Self::FitScreen => ICON_FIT_SCREEN,
            Self::FolderOpen => ICON_FOLDER_OPEN,
            Self::Hub => ICON_HUB,
            Self::Layers => ICON_LAYERS,
            Self::LightMode => ICON_LIGHT_MODE,
            Self::Lock => ICON_LOCK,
            Self::LockOpen => ICON_LOCK_OPEN,
            Self::MenuBook => ICON_MENU_BOOK,
            Self::Monitoring => ICON_MONITORING,
            Self::PhotoLibrary => ICON_PHOTO_LIBRARY,
            Self::Refresh => ICON_REFRESH,
            Self::Remove => ICON_REMOVE,
            Self::RestartAlt => ICON_RESTART_ALT,
            Self::Save => ICON_SAVE,
            Self::Schedule => ICON_SCHEDULE,
            Self::Settings => ICON_SETTINGS,
            Self::SidebarClosed => ICON_SIDEBAR_CLOSED,
            Self::SidebarOpen => ICON_SIDEBAR_OPEN,
            Self::Straighten => ICON_STRAIGHTEN,
            Self::Terminal => ICON_TERMINAL,
            Self::VerifiedUser => ICON_VERIFIED_USER,
            Self::ViewInAr => ICON_VIEW_IN_AR,
            Self::Visibility => ICON_VISIBILITY,
            Self::Warning => ICON_WARNING,
            Self::ZoomIn => ICON_ZOOM_IN,
            Self::ZoomOut => ICON_ZOOM_OUT,
        }
    }
}

pub fn ui_icon<'a>(icon: UiIcon, size: f32, color: Color) -> Svg<'a> {
    Svg::new(Handle::from_memory(icon.bytes()))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(move |_, _| svg::Style { color: Some(color) })
}
