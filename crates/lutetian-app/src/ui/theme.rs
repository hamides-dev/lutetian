use crate::ui::icons::{UiIcon, ui_icon};
use iced::alignment::Horizontal;
use iced::font::{Family, Weight};
use iced::widget::scrollable::{self, Rail, Scrollbar, Scroller};
use iced::widget::text::{Alignment as TextAlignment, Shaping};
use iced::widget::{
    Space, Stack, button, column, container, mouse_area, row, text as wtext, text_input, toggler,
};
use iced::{Alignment, Background, Border, Color, Element, Font, Length, Padding, Theme};
use lutetian_core::UiThemeMode;

// Bundled open-source fonts (matching E:\langhian)
const INTER_REGULAR: &[u8] = include_bytes!("../../../../assets/fonts/Inter-Regular.ttf");
const INTER_MEDIUM: &[u8] = include_bytes!("../../../../assets/fonts/Inter-Medium.ttf");
const INTER_SEMIBOLD: &[u8] = include_bytes!("../../../../assets/fonts/Inter-SemiBold.ttf");
const INTER_BOLD: &[u8] = include_bytes!("../../../../assets/fonts/Inter-Bold.ttf");
const VAZIR_REGULAR: &[u8] = include_bytes!("../../../../assets/fonts/Vazirmatn-Regular.ttf");
const VAZIR_MEDIUM: &[u8] = include_bytes!("../../../../assets/fonts/Vazirmatn-Medium.ttf");
const VAZIR_BOLD: &[u8] = include_bytes!("../../../../assets/fonts/Vazirmatn-Bold.ttf");

pub fn bundled_fonts() -> [&'static [u8]; 7] {
    [
        INTER_REGULAR,
        INTER_MEDIUM,
        INTER_SEMIBOLD,
        INTER_BOLD,
        VAZIR_REGULAR,
        VAZIR_MEDIUM,
        VAZIR_BOLD,
    ]
}

#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    pub bg_app: Color,
    pub bg_sidebar: Color,
    pub bg_surface: Color,
    pub bg_surface_hover: Color,
    pub bg_elevated: Color,
    pub bg_input: Color,
    pub border_subtle: Color,
    pub border_focus: Color,
    pub divider: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_soft: Color,
    pub accent_text: Color,
    pub success: Color,
    pub success_soft: Color,
    pub warning: Color,
    pub warning_soft: Color,
    pub danger: Color,
    pub danger_soft: Color,
    pub scrollbar_thumb: Color,
    pub scrollbar_thumb_hover: Color,
}

pub const DARK_PALETTE: ThemePalette = ThemePalette {
    bg_app: Color::from_rgb(0.059, 0.055, 0.078),
    bg_sidebar: Color::from_rgb(0.043, 0.039, 0.059),
    bg_surface: Color::from_rgb(0.086, 0.078, 0.114),
    bg_surface_hover: Color::from_rgb(0.118, 0.106, 0.153),
    bg_elevated: Color::from_rgb(0.110, 0.098, 0.141),
    bg_input: Color::from_rgb(0.051, 0.047, 0.071),
    border_subtle: Color::from_rgba(0.84, 0.78, 0.90, 0.095),
    border_focus: Color::from_rgba(0.651, 0.545, 0.745, 0.65),
    divider: Color::from_rgba(0.84, 0.78, 0.90, 0.065),
    text_primary: Color::from_rgb(0.945, 0.937, 0.961),
    text_secondary: Color::from_rgb(0.678, 0.651, 0.722),
    text_muted: Color::from_rgb(0.478, 0.447, 0.529),
    accent: Color::from_rgb(0.467, 0.388, 0.549),
    accent_hover: Color::from_rgb(0.545, 0.463, 0.631),
    accent_soft: Color::from_rgba(0.467, 0.388, 0.549, 0.22),
    accent_text: Color::from_rgb(0.788, 0.722, 0.831),
    success: Color::from_rgb(0.243, 0.804, 0.588),
    success_soft: Color::from_rgba(0.243, 0.804, 0.588, 0.15),
    warning: Color::from_rgb(0.965, 0.714, 0.282),
    warning_soft: Color::from_rgba(0.965, 0.714, 0.282, 0.15),
    danger: Color::from_rgb(0.949, 0.435, 0.467),
    danger_soft: Color::from_rgba(0.949, 0.435, 0.467, 0.15),
    scrollbar_thumb: Color::from_rgba(0.82, 0.76, 0.88, 0.18),
    scrollbar_thumb_hover: Color::from_rgba(0.82, 0.76, 0.88, 0.34),
};

pub const LIGHT_PALETTE: ThemePalette = ThemePalette {
    bg_app: Color::from_rgb(0.969, 0.961, 0.980),
    bg_sidebar: Color::from_rgb(0.937, 0.922, 0.957),
    bg_surface: Color::from_rgb(1.0, 1.0, 1.0),
    bg_surface_hover: Color::from_rgb(0.953, 0.937, 0.973),
    bg_elevated: Color::from_rgb(0.937, 0.918, 0.961),
    bg_input: Color::from_rgb(0.980, 0.976, 0.988),
    border_subtle: Color::from_rgba(0.220, 0.180, 0.270, 0.115),
    border_focus: Color::from_rgba(0.408, 0.361, 0.455, 0.58),
    divider: Color::from_rgba(0.220, 0.180, 0.270, 0.08),
    text_primary: Color::from_rgb(0.114, 0.098, 0.145),
    text_secondary: Color::from_rgb(0.369, 0.333, 0.424),
    text_muted: Color::from_rgb(0.525, 0.490, 0.580),
    accent: Color::from_rgb(0.408, 0.361, 0.455),
    accent_hover: Color::from_rgb(0.337, 0.294, 0.384),
    accent_soft: Color::from_rgba(0.408, 0.361, 0.455, 0.13),
    accent_text: Color::from_rgb(0.353, 0.302, 0.408),
    success: Color::from_rgb(0.067, 0.592, 0.416),
    success_soft: Color::from_rgba(0.067, 0.592, 0.416, 0.12),
    warning: Color::from_rgb(0.784, 0.471, 0.055),
    warning_soft: Color::from_rgba(0.784, 0.471, 0.055, 0.12),
    danger: Color::from_rgb(0.812, 0.188, 0.220),
    danger_soft: Color::from_rgba(0.812, 0.188, 0.220, 0.11),
    scrollbar_thumb: Color::from_rgba(0.220, 0.180, 0.270, 0.20),
    scrollbar_thumb_hover: Color::from_rgba(0.220, 0.180, 0.270, 0.36),
};

pub fn palette_for(mode: UiThemeMode) -> ThemePalette {
    match mode {
        UiThemeMode::Dark => DARK_PALETTE,
        UiThemeMode::Light => LIGHT_PALETTE,
    }
}

pub fn transparent_iced_theme_for(mode: UiThemeMode) -> Theme {
    let p = palette_for(mode);
    Theme::custom(
        match mode {
            UiThemeMode::Dark => "Lutetian Velvet Plum Dark".to_string(),
            UiThemeMode::Light => "Lutetian Velvet Plum Light".to_string(),
        },
        iced::theme::Palette {
            background: Color::TRANSPARENT,
            text: p.text_primary,
            primary: p.accent,
            success: p.success,
            warning: p.warning,
            danger: p.danger,
        },
    )
}

fn normalize_weight(weight: Weight) -> Weight {
    match weight {
        Weight::Semibold => Weight::Medium,
        other => other,
    }
}

pub fn font_ui_default() -> Font {
    font_for_ui(Weight::Normal)
}

pub fn font_for_ui(weight: Weight) -> Font {
    Font {
        family: Family::Name("Inter"),
        weight: normalize_weight(weight),
        ..Font::DEFAULT
    }
}

pub fn aligned_text<'a, M: 'a>(
    content: impl Into<std::borrow::Cow<'a, str>>,
    size: f32,
    font: Font,
    color: Option<Color>,
) -> Element<'a, M> {
    let s: std::borrow::Cow<'a, str> = content.into();
    let mut t = wtext(s)
        .size(size)
        .font(font)
        .align_x(TextAlignment::Default)
        .width(Length::Shrink)
        .shaping(Shaping::Advanced);
    if let Some(c) = color {
        t = t.color(c);
    }
    container(t)
        .width(Length::Fill)
        .align_x(Horizontal::Left)
        .into()
}

pub fn square_icon_button<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    size: f32,
    on_press: M,
    style: impl Fn(&Theme, button::Status) -> button::Style + 'a,
) -> button::Button<'a, M> {
    button(
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .on_press(on_press)
    .width(Length::Fixed(size))
    .height(Length::Fixed(size))
    .padding(0)
    .style(style)
}

pub fn wrap_frameless_resize_frame<'a, M: Clone + 'a>(
    inner: Element<'a, M>,
    mode: UiThemeMode,
    radius: f32,
    is_maximized: bool,
    on_resize: impl Fn(iced::window::Direction) -> M,
) -> Element<'a, M> {
    let eff_radius = if is_maximized { 0.0 } else { radius };
    let framed: Element<'a, M> = container(inner)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(if is_maximized { 0.0 } else { 1.0 })
        .style(app_shell_style(mode, eff_radius))
        .into();

    if is_maximized {
        return framed;
    }

    let top_edge = 4.0_f32;
    let edge = 5.0_f32;
    let corner = 10.0_f32;

    let handle = |dir: iced::window::Direction,
                  w: Length,
                  h: Length,
                  cursor: iced::mouse::Interaction|
     -> Element<'a, M> {
        mouse_area(Space::new().width(w).height(h))
            .on_press(on_resize(dir))
            .interaction(cursor)
            .into()
    };

    let top_row = row![
        handle(
            iced::window::Direction::NorthWest,
            Length::Fixed(corner),
            Length::Fixed(top_edge),
            iced::mouse::Interaction::ResizingDiagonallyDown,
        ),
        handle(
            iced::window::Direction::North,
            Length::Fill,
            Length::Fixed(top_edge),
            iced::mouse::Interaction::ResizingVertically,
        ),
        handle(
            iced::window::Direction::NorthEast,
            Length::Fixed(corner),
            Length::Fixed(top_edge),
            iced::mouse::Interaction::ResizingDiagonallyUp,
        ),
    ]
    .width(Length::Fill);

    let middle_row = row![
        handle(
            iced::window::Direction::West,
            Length::Fixed(edge),
            Length::Fill,
            iced::mouse::Interaction::ResizingHorizontally,
        ),
        Space::new().width(Length::Fill).height(Length::Fill),
        handle(
            iced::window::Direction::East,
            Length::Fixed(edge),
            Length::Fill,
            iced::mouse::Interaction::ResizingHorizontally,
        ),
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    let bottom_row = row![
        handle(
            iced::window::Direction::SouthWest,
            Length::Fixed(corner),
            Length::Fixed(edge),
            iced::mouse::Interaction::ResizingDiagonallyUp,
        ),
        handle(
            iced::window::Direction::South,
            Length::Fill,
            Length::Fixed(edge),
            iced::mouse::Interaction::ResizingVertically,
        ),
        handle(
            iced::window::Direction::SouthEast,
            Length::Fixed(corner),
            Length::Fixed(edge),
            iced::mouse::Interaction::ResizingDiagonallyDown,
        ),
    ]
    .width(Length::Fill);

    let resize_overlay: Element<'a, M> = column![top_row, middle_row, bottom_row]
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

    Stack::with_children(vec![framed, resize_overlay])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub fn sleek_scrollable<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    mode: UiThemeMode,
) -> scrollable::Scrollable<'a, M> {
    scrollable::Scrollable::new(content)
        .direction(scrollable::Direction::Vertical(
            Scrollbar::new().width(5).scroller_width(4).spacing(8),
        ))
        .style(sleek_scrollable_style(mode))
}

pub fn sleek_scrollable_style(
    mode: UiThemeMode,
) -> impl Fn(&Theme, scrollable::Status) -> scrollable::Style {
    move |theme, status| {
        let p = palette_for(mode);
        let base = scrollable::default(theme, status);
        let is_hovered = matches!(
            status,
            scrollable::Status::Hovered { .. } | scrollable::Status::Dragged { .. }
        );
        let thumb_color = if is_hovered {
            p.scrollbar_thumb_hover
        } else {
            p.scrollbar_thumb
        };

        scrollable::Style {
            container: container::Style::default(),
            vertical_rail: Rail {
                background: Some(Background::Color(Color::TRANSPARENT)),
                border: Border {
                    radius: 999.0.into(),
                    ..Default::default()
                },
                scroller: Scroller {
                    background: Background::Color(thumb_color),
                    border: Border {
                        radius: 999.0.into(),
                        ..Default::default()
                    },
                },
            },
            horizontal_rail: Rail {
                background: Some(Background::Color(Color::TRANSPARENT)),
                border: Border::default(),
                scroller: Scroller {
                    background: Background::Color(thumb_color),
                    border: Border {
                        radius: 999.0.into(),
                        ..Default::default()
                    },
                },
            },
            gap: None,
            auto_scroll: base.auto_scroll,
        }
    }
}

pub fn card_style(mode: UiThemeMode, radius: f32) -> impl Fn(&Theme) -> container::Style {
    move |_| {
        let p = palette_for(mode);
        container::Style {
            background: Some(Background::Color(p.bg_surface)),
            border: Border {
                color: p.border_subtle,
                width: 1.0,
                radius: radius.into(),
            },
            text_color: Some(p.text_primary),
            ..Default::default()
        }
    }
}

pub fn elevated_box_style(mode: UiThemeMode, radius: f32) -> impl Fn(&Theme) -> container::Style {
    move |_| {
        let p = palette_for(mode);
        container::Style {
            background: Some(Background::Color(p.bg_elevated)),
            border: Border {
                color: p.border_subtle,
                width: 1.0,
                radius: radius.into(),
            },
            text_color: Some(p.text_primary),
            ..Default::default()
        }
    }
}

pub fn app_shell_style(mode: UiThemeMode, radius: f32) -> impl Fn(&Theme) -> container::Style {
    move |_| {
        let p = palette_for(mode);
        container::Style {
            background: Some(Background::Color(p.bg_app)),
            text_color: Some(p.text_primary),
            border: iced::Border {
                color: p.border_subtle,
                width: if radius > 0.0 { 1.0 } else { 0.0 },
                radius: radius.into(),
            },
            ..Default::default()
        }
    }
}

pub fn rounded_sidebar_style(
    mode: UiThemeMode,
    bottom_radius: f32,
) -> impl Fn(&Theme) -> container::Style {
    move |_| {
        let p = palette_for(mode);
        container::Style {
            background: Some(Background::Color(p.bg_sidebar)),
            text_color: Some(p.text_primary),
            border: iced::Border {
                color: p.border_subtle,
                width: 0.0,
                radius: iced::border::Radius {
                    top_left: 0.0,
                    top_right: 0.0,
                    bottom_right: 0.0,
                    bottom_left: bottom_radius,
                },
            },
            ..Default::default()
        }
    }
}

pub fn rounded_titlebar_style(
    mode: UiThemeMode,
    top_radius: f32,
) -> impl Fn(&Theme) -> container::Style {
    move |_| {
        let p = palette_for(mode);
        container::Style {
            background: Some(Background::Color(p.bg_sidebar)),
            text_color: Some(p.text_primary),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: iced::border::Radius {
                    top_left: top_radius,
                    top_right: top_radius,
                    bottom_right: 0.0,
                    bottom_left: 0.0,
                },
            },
            ..Default::default()
        }
    }
}

pub fn selectable_card_style(
    mode: UiThemeMode,
    selected: bool,
    radius: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let p = palette_for(mode);
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let bg = if selected {
            p.accent_soft
        } else if hovered {
            p.bg_surface_hover
        } else {
            p.bg_surface
        };
        let border_color = if selected {
            p.accent
        } else if hovered {
            p.border_focus
        } else {
            p.border_subtle
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: p.text_primary,
            border: Border {
                color: border_color,
                width: if selected { 1.4 } else { 1.0 },
                radius: radius.into(),
            },
            ..Default::default()
        }
    }
}

pub fn primary_button_style(mode: UiThemeMode) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let p = palette_for(mode);
        let bg = match status {
            button::Status::Hovered | button::Status::Pressed => p.accent_hover,
            button::Status::Disabled => p.accent_soft,
            _ => p.accent,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: Color::WHITE,
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

pub fn pill_button_style(mode: UiThemeMode) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let p = palette_for(mode);
        let bg = match status {
            button::Status::Hovered | button::Status::Pressed => p.bg_surface_hover,
            _ => p.bg_elevated,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: p.text_primary,
            border: Border {
                color: p.border_subtle,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }
}

pub fn ghost_button_style(
    mode: UiThemeMode,
    active: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let p = palette_for(mode);
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let bg = if active {
            p.accent_soft
        } else if hovered {
            p.bg_elevated
        } else {
            Color::TRANSPARENT
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: if active {
                p.accent_text
            } else if hovered {
                p.text_primary
            } else {
                p.text_secondary
            },
            border: Border {
                color: if active {
                    p.border_focus
                } else {
                    Color::TRANSPARENT
                },
                width: if active { 1.0 } else { 0.0 },
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }
}

pub fn window_close_button_style(
    mode: UiThemeMode,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let p = palette_for(mode);
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(Background::Color(if hovered {
                p.danger
            } else {
                Color::TRANSPARENT
            })),
            text_color: if hovered {
                Color::WHITE
            } else {
                p.text_secondary
            },
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

pub fn danger_button_style(mode: UiThemeMode) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let p = palette_for(mode);
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(Background::Color(if hovered {
                Color::from_rgba(0.95, 0.36, 0.36, 0.25)
            } else {
                p.danger_soft
            })),
            text_color: p.danger,
            border: Border {
                color: Color::from_rgba(0.95, 0.36, 0.36, 0.32),
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }
}

pub fn nav_button_style(
    mode: UiThemeMode,
    active: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let p = palette_for(mode);
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let bg = if active {
            p.bg_elevated
        } else if hovered {
            p.bg_surface_hover
        } else {
            Color::TRANSPARENT
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: if active || hovered {
                p.text_primary
            } else {
                p.text_secondary
            },
            border: Border {
                color: if active {
                    p.border_subtle
                } else {
                    Color::TRANSPARENT
                },
                width: if active { 1.0 } else { 0.0 },
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }
}

pub fn badge_style(bg: Color, fg: Color, border_col: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(bg)),
        text_color: Some(fg),
        border: Border {
            color: border_col,
            width: 1.0,
            radius: 999.0.into(),
        },
        ..Default::default()
    }
}

pub fn sleek_input_style(
    mode: UiThemeMode,
) -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
    move |_, status| {
        let p = palette_for(mode);
        let focused = matches!(status, text_input::Status::Focused { .. });
        let hovered = matches!(status, text_input::Status::Hovered);
        text_input::Style {
            background: Background::Color(p.bg_input),
            border: Border {
                color: if focused || hovered {
                    p.border_focus
                } else {
                    p.border_subtle
                },
                width: 1.0,
                radius: 8.0.into(),
            },
            icon: p.text_muted,
            placeholder: p.text_muted,
            value: p.text_primary,
            selection: p.accent_soft,
        }
    }
}

pub fn sleek_toggler_style(
    mode: UiThemeMode,
) -> impl Fn(&Theme, toggler::Status) -> toggler::Style {
    move |_, status| {
        let p = palette_for(mode);
        let is_toggled = match status {
            toggler::Status::Active { is_toggled }
            | toggler::Status::Hovered { is_toggled }
            | toggler::Status::Disabled { is_toggled } => is_toggled,
        };
        let off_bg = match mode {
            UiThemeMode::Dark => Color::from_rgba(1.0, 1.0, 1.0, 0.14),
            UiThemeMode::Light => Color::from_rgba(0.06, 0.09, 0.16, 0.18),
        };
        toggler::Style {
            background: Background::Color(if is_toggled { p.accent } else { off_bg }),
            background_border_width: 0.0,
            background_border_color: Color::TRANSPARENT,
            foreground: Background::Color(Color::WHITE),
            foreground_border_width: 0.0,
            foreground_border_color: Color::TRANSPARENT,
            border_radius: Some(999.0.into()),
            padding_ratio: 0.14,
            text_color: None,
        }
    }
}

pub fn hairline_divider<'a, M: 'a>(mode: UiThemeMode) -> Element<'a, M> {
    let p = palette_for(mode);
    container(Space::new().width(Length::Fill).height(1))
        .style(move |_| container::Style {
            background: Some(Background::Color(p.divider)),
            ..Default::default()
        })
        .into()
}

pub fn sleek_row<'a, M: 'a>(
    mode: UiThemeMode,
    title: impl Into<String>,
    subtitle: Option<String>,
    control: Element<'a, M>,
    ui_font: Font,
    ui_font_semi: Font,
) -> Element<'a, M> {
    let p = palette_for(mode);
    let mut label_col = column![aligned_text(
        title.into(),
        13.0,
        ui_font_semi,
        Some(p.text_primary),
    )]
    .spacing(3)
    .align_x(Horizontal::Left)
    .width(Length::Fill);

    if let Some(sub) = subtitle {
        label_col = label_col.push(aligned_text(sub, 12.0, ui_font, Some(p.text_secondary)));
    }

    container(
        row![label_col, control]
            .spacing(18)
            .align_y(Alignment::Center)
            .width(Length::Fill),
    )
    .padding(Padding::from([14, 18]))
    .width(Length::Fill)
    .into()
}

pub fn theme_preview_card_content<'a, M: 'a>(
    target_mode: UiThemeMode,
    is_selected: bool,
    title: String,
    desc: String,
    current_mode: UiThemeMode,
    ui_font: Font,
    ui_font_bold: Font,
) -> Element<'a, M> {
    let preview = palette_for(target_mode);
    let current = palette_for(current_mode);

    let mini_sidebar = container(
        column![
            container(Space::new().width(22).height(5)).style(move |_| container::Style {
                background: Some(Background::Color(preview.accent)),
                border: Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            container(Space::new().width(18).height(4)).style(move |_| container::Style {
                background: Some(Background::Color(preview.border_subtle)),
                border: Border {
                    radius: 2.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            container(Space::new().width(16).height(4)).style(move |_| container::Style {
                background: Some(Background::Color(preview.border_subtle)),
                border: Border {
                    radius: 2.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        ]
        .spacing(4),
    )
    .padding(6)
    .width(Length::Fixed(34.0))
    .height(Length::Fill)
    .style(move |_| container::Style {
        background: Some(Background::Color(preview.bg_sidebar)),
        ..Default::default()
    });

    let mini_body = container(
        column![
            container(Space::new().width(Length::Fill).height(12)).style(move |_| {
                container::Style {
                    background: Some(Background::Color(preview.bg_surface)),
                    border: Border {
                        color: preview.border_subtle,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            }),
            row![
                container(Space::new().width(Length::FillPortion(1)).height(16)).style(move |_| {
                    container::Style {
                        background: Some(Background::Color(preview.bg_surface)),
                        border: Border {
                            color: preview.border_subtle,
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }
                }),
                container(Space::new().width(Length::FillPortion(1)).height(16)).style(move |_| {
                    container::Style {
                        background: Some(Background::Color(preview.accent_soft)),
                        border: Border {
                            color: preview.border_focus,
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }
                }),
            ]
            .spacing(4),
        ]
        .spacing(4),
    )
    .padding(6)
    .width(Length::Fill)
    .height(Length::Fill);

    let mini_window = container(row![mini_sidebar, mini_body].height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fixed(48.0))
        .style(move |_| container::Style {
            background: Some(Background::Color(preview.bg_app)),
            border: Border {
                color: preview.border_subtle,
                width: 1.0,
                radius: 7.0.into(),
            },
            ..Default::default()
        });

    let icon_kind = match target_mode {
        UiThemeMode::Dark => UiIcon::DarkMode,
        UiThemeMode::Light => UiIcon::LightMode,
    };
    let title_row = row![
        ui_icon(
            icon_kind,
            16.0,
            if is_selected {
                current.accent_text
            } else {
                current.text_secondary
            },
        ),
        aligned_text(title, 14.0, ui_font_bold, Some(current.text_primary)),
        if is_selected {
            Element::from(ui_icon(UiIcon::CheckCircle, 16.0, current.accent))
        } else {
            Space::new().width(16).height(16).into()
        },
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    column![
        mini_window,
        Space::new().height(4),
        title_row,
        aligned_text(desc, 12.0, ui_font, Some(current.text_secondary)),
    ]
    .spacing(6)
    .align_x(Horizontal::Left)
    .height(Length::Fixed(122.0))
    .into()
}
