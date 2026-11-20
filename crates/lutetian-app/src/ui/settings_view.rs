use crate::ui::dashboard::{DashboardMsg, DashboardState, SettingsSubTab, page_header};
use crate::ui::icons::{UiIcon, ui_icon};
use crate::ui::theme::{
    ThemePalette, card_style, ghost_button_style, hairline_divider, pill_button_style,
    primary_button_style, selectable_card_style, sleek_input_style, sleek_row, sleek_scrollable,
    sleek_toggler_style, theme_preview_card_content,
};
use iced::widget::{
    Space, button, column, container, progress_bar, row, text, text_input, toggler,
};
use iced::{Alignment, Element, Font, Length};
use lutetian_core::{AppConfig, UiThemeMode};

#[allow(clippy::too_many_arguments)]
pub fn view_settings_tab<'a>(
    state: &'a DashboardState,
    config: &'a AppConfig,
    mode: UiThemeMode,
    p: ThemePalette,
    ui_font: Font,
    ui_font_semi: Font,
    ui_font_bold: Font,
) -> Element<'a, DashboardMsg> {
    let header = page_header(
        "Settings",
        Some(
            "Configure Velvet Plum appearance, Antigravity MCP bridge, and automatic updates."
                .to_string(),
        ),
        None,
        p,
        ui_font,
        ui_font_bold,
    );

    let subtab_bar = row![
        subtab_btn(
            "Bridge & Plugin",
            SettingsSubTab::General,
            state.settings_subtab,
            mode,
            p,
            ui_font_semi
        ),
        subtab_btn(
            "Appearance",
            SettingsSubTab::Appearance,
            state.settings_subtab,
            mode,
            p,
            ui_font_semi
        ),
        subtab_btn(
            "Updates",
            SettingsSubTab::Updates,
            state.settings_subtab,
            mode,
            p,
            ui_font_semi
        ),
    ]
    .spacing(8);

    let body = match state.settings_subtab {
        SettingsSubTab::General => view_settings_general(state, mode, p, ui_font, ui_font_semi),
        SettingsSubTab::Appearance => view_settings_appearance(config, mode, ui_font, ui_font_bold),
        SettingsSubTab::Updates => {
            view_settings_updates(state, config, mode, p, ui_font, ui_font_semi)
        }
    };

    sleek_scrollable(
        column![header, subtab_bar, body]
            .spacing(16)
            .width(Length::Fill),
        mode,
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn subtab_btn<'a>(
    label: &'a str,
    tab: SettingsSubTab,
    active: SettingsSubTab,
    mode: UiThemeMode,
    p: ThemePalette,
    ui_font_semi: Font,
) -> Element<'a, DashboardMsg> {
    let is_active = tab == active;
    button(
        text(label)
            .size(12.5)
            .font(ui_font_semi)
            .color(if is_active {
                p.accent_text
            } else {
                p.text_secondary
            }),
    )
    .on_press(DashboardMsg::SwitchSettingsSubTab(tab))
    .padding([7, 14])
    .style(ghost_button_style(mode, is_active))
    .into()
}

fn view_settings_general<'a>(
    state: &'a DashboardState,
    mode: UiThemeMode,
    p: ThemePalette,
    ui_font: Font,
    ui_font_semi: Font,
) -> Element<'a, DashboardMsg> {
    let bridge_card = container(column![
        sleek_row(
            mode,
            "Localhost Bridge Port",
            Some(format!(
                "Active HTTP/WebSocket bridge listening on 127.0.0.1:{}",
                state.bridge_port
            )),
            row![
                text_input("47821", &state.bridge_port_buf)
                    .on_input(DashboardMsg::BridgePortChanged)
                    .size(12.5)
                    .padding([7, 10])
                    .width(Length::Fixed(96.0))
                    .font(Font::MONOSPACE)
                    .style(sleek_input_style(mode)),
                button(text("Save Port").size(12).font(ui_font_semi))
                    .on_press(DashboardMsg::ApplyBridgePort)
                    .padding([7, 13])
                    .style(pill_button_style(mode)),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
            .into(),
            ui_font,
            ui_font_semi,
        ),
        hairline_divider(mode),
        sleek_row(
            mode,
            "Google Antigravity MCP Plugin",
            Some(
                "Copy ready-to-use MCP config JSON or open the antigravity-plugin folder"
                    .to_string()
            ),
            row![
                button(
                    row![
                        ui_icon(UiIcon::Copy, 14.0, p.text_primary),
                        text("Copy MCP Config").size(12).font(ui_font_semi),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .on_press(DashboardMsg::CopyMcpConfigJson)
                .padding([7, 13])
                .style(pill_button_style(mode)),
                button(
                    row![
                        ui_icon(UiIcon::FolderOpen, 14.0, p.text_primary),
                        text("Open Plugin Folder").size(12).font(ui_font_semi),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .on_press(DashboardMsg::OpenPluginFolder)
                .padding([7, 13])
                .style(pill_button_style(mode)),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
            .into(),
            ui_font,
            ui_font_semi,
        ),
        hairline_divider(mode),
        sleek_row(
            mode,
            "New Empty Environment Project",
            Some("Reset workspace to blank Stage 1 ingestion state".to_string()),
            button(
                row![
                    ui_icon(UiIcon::Add, 14.0, p.text_primary),
                    text("New Blank Project").size(12).font(ui_font_semi),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .on_press(DashboardMsg::NewEmptyProject)
            .padding([7, 13])
            .style(pill_button_style(mode))
            .into(),
            ui_font,
            ui_font_semi,
        ),
    ])
    .width(Length::Fill)
    .style(card_style(mode, 10.0));

    bridge_card.into()
}

fn view_settings_appearance<'a>(
    config: &'a AppConfig,
    mode: UiThemeMode,
    ui_font: Font,
    ui_font_bold: Font,
) -> Element<'a, DashboardMsg> {
    let dark_selected = config.theme == UiThemeMode::Dark;
    let light_selected = config.theme == UiThemeMode::Light;

    row![
        button(theme_preview_card_content(
            UiThemeMode::Dark,
            dark_selected,
            "Midnight Dark".to_string(),
            "Deep Velvet Plum charcoal surfaces for long studio sessions.".to_string(),
            mode,
            ui_font,
            ui_font_bold,
        ))
        .on_press(DashboardMsg::ThemeChanged(UiThemeMode::Dark))
        .padding(16)
        .width(Length::FillPortion(1))
        .style(selectable_card_style(mode, dark_selected, 12.0)),
        button(theme_preview_card_content(
            UiThemeMode::Light,
            light_selected,
            "Daylight Light".to_string(),
            "Crisp lavender-tinted studio paper for high-ambient rooms.".to_string(),
            mode,
            ui_font,
            ui_font_bold,
        ))
        .on_press(DashboardMsg::ThemeChanged(UiThemeMode::Light))
        .padding(16)
        .width(Length::FillPortion(1))
        .style(selectable_card_style(mode, light_selected, 12.0)),
    ]
    .spacing(14)
    .width(Length::Fill)
    .into()
}

fn view_settings_updates<'a>(
    state: &'a DashboardState,
    config: &'a AppConfig,
    mode: UiThemeMode,
    p: ThemePalette,
    ui_font: Font,
    ui_font_semi: Font,
) -> Element<'a, DashboardMsg> {
    let mut update_desc = format!("Current version: v{}", env!("CARGO_PKG_VERSION"));
    let mut action_items: Vec<Element<'a, DashboardMsg>> = vec![
        button(
            row![
                ui_icon(UiIcon::Refresh, 14.0, p.text_primary),
                text(if state.update_checking {
                    "Checking..."
                } else {
                    "Check for Updates"
                })
                .size(12)
                .font(ui_font_semi),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .on_press(DashboardMsg::CheckUpdatesNow)
        .padding([7, 13])
        .style(pill_button_style(mode))
        .into(),
    ];

    if let Some(res) = &state.update_status {
        if res.has_update {
            update_desc = format!("{update_desc}  ·  Available: v{}", res.latest_version);
            let dl_label = if state.update_downloading {
                match state.update_download_progress {
                    Some(pct) => format!("Downloading {pct}%"),
                    None => "Downloading...".to_string(),
                }
            } else {
                "Download & Install".to_string()
            };
            action_items.push(
                button(
                    row![
                        ui_icon(UiIcon::Download, 14.0, iced::Color::WHITE),
                        text(dl_label)
                            .size(12)
                            .font(ui_font_semi)
                            .color(iced::Color::WHITE),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .on_press_maybe(if state.update_downloading {
                    None
                } else {
                    Some(DashboardMsg::DownloadAndInstallUpdate)
                })
                .padding([7, 14])
                .style(primary_button_style(mode))
                .into(),
            );
            action_items.push(
                button(text("Release Notes").size(12).font(ui_font_semi))
                    .on_press(DashboardMsg::OpenReleaseUrl(res.release_url.clone()))
                    .padding([7, 12])
                    .style(pill_button_style(mode))
                    .into(),
            );
        } else {
            update_desc = format!("{update_desc}  ·  Up to date");
        }
    } else if let Some(err) = &state.update_error {
        update_desc = format!("{update_desc}  ·  {err}");
    }

    let mut card_items: Vec<Element<'a, DashboardMsg>> = vec![
        sleek_row(
            mode,
            "Auto-Check for Updates",
            Some(
                "Check GitHub Releases (hamides-dev/lutetian) in background on launch".to_string(),
            ),
            toggler(config.auto_check_updates)
                .on_toggle(DashboardMsg::ToggleAutoCheckUpdates)
                .size(19)
                .style(sleek_toggler_style(mode))
                .into(),
            ui_font,
            ui_font_semi,
        ),
        hairline_divider(mode),
        sleek_row(
            mode,
            "Include Pre-releases / Nightly",
            Some("Track nightly builds and release candidates".to_string()),
            toggler(config.include_prereleases)
                .on_toggle(DashboardMsg::ToggleIncludePrereleases)
                .size(19)
                .style(sleek_toggler_style(mode))
                .into(),
            ui_font,
            ui_font_semi,
        ),
        hairline_divider(mode),
        sleek_row(
            mode,
            "Lutetian Updates",
            Some(update_desc),
            row(action_items)
                .spacing(8)
                .align_y(Alignment::Center)
                .into(),
            ui_font,
            ui_font_semi,
        ),
    ];

    if state.update_downloading {
        let pct = state.update_download_progress.unwrap_or(0) as f32;
        card_items.push(
            container(progress_bar(0.0..=100.0, pct).girth(4))
                .padding([0, 16])
                .width(Length::Fill)
                .into(),
        );
        card_items.push(Space::new().height(10).into());
    }

    container(column(card_items))
        .width(Length::Fill)
        .style(card_style(mode, 10.0))
        .into()
}
