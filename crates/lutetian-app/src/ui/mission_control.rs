use crate::ui::dashboard::{DashboardMsg, page_header};
use crate::ui::icons::{UiIcon, ui_icon};
use crate::ui::theme::{
    ThemePalette, aligned_text, badge_style, card_style, elevated_box_style, hairline_divider,
    pill_button_style, primary_button_style, sleek_scrollable,
};
use iced::widget::{Space, button, column, container, progress_bar, row, text};
use iced::{Alignment, Element, Font, Length};
use lutetian_core::{
    ActivityLevel, ComplianceStatus, ExecutionTask, LutetianProject, TaskState, UiThemeMode,
};

#[allow(clippy::too_many_arguments)]
pub fn view_mission_control<'a>(
    project: &'a LutetianProject,
    bridge_port: u16,
    mode: UiThemeMode,
    p: ThemePalette,
    ui_font: Font,
    ui_font_semi: Font,
    ui_font_bold: Font,
) -> Element<'a, DashboardMsg> {
    let hb = &project.telemetry.heartbeat;

    let run_val_btn = button(
        row![
            ui_icon(UiIcon::VerifiedUser, 14.0, iced::Color::WHITE),
            text("Run Deterministic Scale Validator")
                .size(12)
                .font(ui_font_semi)
                .color(iced::Color::WHITE),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .on_press(DashboardMsg::RunScaleValidation)
    .padding([8, 14])
    .style(primary_button_style(mode));

    let header = page_header(
        "Mission Control · Live Agent Telemetry & Compliance",
        Some("Real-time visibility into Antigravity's active stage, 3-state execution queue, deterministic math rule compliance, and live tool stream.".to_string()),
        Some(run_val_btn.into()),
        p,
        ui_font,
        ui_font_bold,
    );

    // 1. Live Heartbeat Card
    let heartbeat_card = container(
        column![
            row![
                ui_icon(
                    UiIcon::Hub,
                    18.0,
                    if hb.connected {
                        p.success
                    } else {
                        p.accent_text
                    }
                ),
                text(&hb.agent_name)
                    .size(14.5)
                    .font(ui_font_bold)
                    .color(p.text_primary),
                container(
                    text(if hb.connected {
                        format!("LIVE MCP · :{}", bridge_port)
                    } else {
                        format!("LISTENING :{}", bridge_port)
                    })
                    .size(11)
                    .font(ui_font_semi)
                    .color(if hb.connected {
                        p.success
                    } else {
                        p.accent_text
                    }),
                )
                .padding([3, 9])
                .style(badge_style(
                    if hb.connected {
                        p.success_soft
                    } else {
                        p.accent_soft
                    },
                    if hb.connected {
                        p.success
                    } else {
                        p.accent_text
                    },
                    p.border_subtle,
                )),
                Space::new().width(Length::Fill),
                text(format!(
                    "Stage {} · Skill: {} · Tool: {}",
                    hb.active_stage.number(),
                    hb.active_skill,
                    hb.active_tool
                ))
                .size(11.5)
                .font(Font::MONOSPACE)
                .color(p.text_secondary),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                aligned_text(
                    format!("Current Action: {}", hb.current_action),
                    12.5,
                    ui_font,
                    Some(p.text_primary),
                ),
                text(format!("{}%", hb.progress_percent))
                    .size(12)
                    .font(Font::MONOSPACE)
                    .color(p.accent_text),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            progress_bar(0.0..=100.0, hb.progress_percent as f32).girth(5),
        ]
        .spacing(10),
    )
    .padding(16)
    .width(Length::Fill)
    .style(card_style(mode, 10.0));

    // 2. 3-Column Execution Pipeline Board (Completed | In Progress | Upcoming)
    let completed: Vec<&ExecutionTask> = project
        .telemetry
        .tasks
        .iter()
        .filter(|t| t.status == TaskState::Completed)
        .collect();
    let in_progress: Vec<&ExecutionTask> = project
        .telemetry
        .tasks
        .iter()
        .filter(|t| t.status == TaskState::InProgress)
        .collect();
    let upcoming: Vec<&ExecutionTask> = project
        .telemetry
        .tasks
        .iter()
        .filter(|t| t.status == TaskState::Upcoming)
        .collect();

    let board = row![
        pipeline_column(
            "Completed",
            UiIcon::CheckCircle,
            p.success,
            p.success_soft,
            &completed,
            mode,
            p,
            ui_font,
            ui_font_semi,
            ui_font_bold,
        ),
        pipeline_column(
            "In Progress",
            UiIcon::Bolt,
            p.accent_text,
            p.accent_soft,
            &in_progress,
            mode,
            p,
            ui_font,
            ui_font_semi,
            ui_font_bold,
        ),
        pipeline_column(
            "Upcoming (Planned)",
            UiIcon::Schedule,
            p.warning,
            p.warning_soft,
            &upcoming,
            mode,
            p,
            ui_font,
            ui_font_semi,
            ui_font_bold,
        ),
    ]
    .spacing(12)
    .width(Length::Fill);

    // 3. Compliance & Rules Matrix
    let mut matrix_col = column![
        row![
            ui_icon(UiIcon::VerifiedUser, 16.0, p.accent_text),
            text("Compliance & Rules Matrix")
                .size(14)
                .font(ui_font_bold)
                .color(p.text_primary),
            Space::new().width(Length::Fill),
            text(format!(
                "{} Errors · {} Warnings",
                project.last_validation.error_count, project.last_validation.warning_count
            ))
            .size(11.5)
            .font(Font::MONOSPACE)
            .color(if project.last_validation.passed {
                p.success
            } else {
                p.danger
            }),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        hairline_divider(mode),
    ]
    .spacing(8)
    .width(Length::Fill);

    for rule in &project.last_validation.compliance_matrix {
        let (bg, fg) = match rule.status {
            ComplianceStatus::Compliant => (p.success_soft, p.success),
            ComplianceStatus::Pending => (p.warning_soft, p.warning),
            ComplianceStatus::Violated => (p.danger_soft, p.danger),
        };
        let mut row_items: Vec<Element<'a, DashboardMsg>> = vec![
            container(
                text(rule.status.badge_text())
                    .size(10.5)
                    .font(ui_font_bold)
                    .color(fg),
            )
            .padding([3, 8])
            .width(Length::Fixed(88.0))
            .center_x(Length::Fill)
            .style(badge_style(bg, fg, p.border_subtle))
            .into(),
            column![
                row![
                    text(&rule.rule_id)
                        .size(11.5)
                        .font(Font::MONOSPACE)
                        .color(p.accent_text),
                    text(&rule.title)
                        .size(12.5)
                        .font(ui_font_semi)
                        .color(p.text_primary),
                    container(
                        text(&rule.category)
                            .size(10.5)
                            .font(ui_font)
                            .color(p.text_muted),
                    )
                    .padding([2, 7])
                    .style(badge_style(
                        p.bg_elevated,
                        p.text_muted,
                        p.border_subtle
                    )),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                aligned_text(rule.evidence.clone(), 11.5, ui_font, Some(p.text_secondary)),
            ]
            .spacing(3)
            .width(Length::Fill)
            .into(),
        ];

        if let Some(eid) = &rule.entity_id {
            row_items.push(
                button(
                    row![
                        ui_icon(UiIcon::Visibility, 13.0, p.text_primary),
                        text(format!("Inspect {eid}")).size(11).font(ui_font_semi),
                    ]
                    .spacing(5)
                    .align_y(Alignment::Center),
                )
                .on_press(DashboardMsg::InspectComplianceEntity(eid.clone()))
                .padding([5, 10])
                .style(pill_button_style(mode))
                .into(),
            );
        }

        matrix_col = matrix_col.push(
            container(
                row(row_items)
                    .spacing(12)
                    .align_y(Alignment::Center)
                    .width(Length::Fill),
            )
            .padding([9, 12])
            .width(Length::Fill)
            .style(elevated_box_style(mode, 8.0)),
        );
    }

    let matrix_card = container(matrix_col)
        .padding(16)
        .width(Length::Fill)
        .style(card_style(mode, 10.0));

    // 4. Live Activity Stream
    let mut log_col = column![
        row![
            ui_icon(UiIcon::Terminal, 16.0, p.accent_text),
            text("Live Activity Stream")
                .size(14)
                .font(ui_font_bold)
                .color(p.text_primary),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        hairline_divider(mode),
    ]
    .spacing(8)
    .width(Length::Fill);

    if project.telemetry.activity_log.is_empty() {
        log_col = log_col.push(aligned_text(
            "No activity recorded yet.",
            12.0,
            ui_font,
            Some(p.text_secondary),
        ));
    } else {
        for entry in project.telemetry.activity_log.iter().take(40) {
            let level_col = match entry.level {
                ActivityLevel::Info => p.accent_text,
                ActivityLevel::Success => p.success,
                ActivityLevel::Warning => p.warning,
                ActivityLevel::Error => p.danger,
            };
            let ts = entry.timestamp.get(11..19).unwrap_or(&entry.timestamp);
            log_col = log_col.push(
                row![
                    text(ts).size(11).font(Font::MONOSPACE).color(p.text_muted),
                    text(format!("[{}:{}]", entry.source, entry.tool))
                        .size(11)
                        .font(Font::MONOSPACE)
                        .color(level_col),
                    aligned_text(entry.message.clone(), 12.0, ui_font, Some(p.text_primary)),
                ]
                .spacing(8)
                .align_y(Alignment::Center)
                .width(Length::Fill),
            );
        }
    }

    let activity_card = container(log_col)
        .padding(16)
        .width(Length::Fill)
        .style(card_style(mode, 10.0));

    sleek_scrollable(
        column![header, heartbeat_card, board, matrix_card, activity_card]
            .spacing(14)
            .width(Length::Fill),
        mode,
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

#[allow(clippy::too_many_arguments)]
fn pipeline_column<'a>(
    title: &'a str,
    icon: UiIcon,
    accent: iced::Color,
    soft_bg: iced::Color,
    tasks: &[&'a ExecutionTask],
    mode: UiThemeMode,
    p: ThemePalette,
    ui_font: Font,
    ui_font_semi: Font,
    ui_font_bold: Font,
) -> Element<'a, DashboardMsg> {
    let mut col = column![
        row![
            ui_icon(icon, 15.0, accent),
            text(title)
                .size(13)
                .font(ui_font_bold)
                .color(p.text_primary),
            Space::new().width(Length::Fill),
            container(
                text(tasks.len().to_string())
                    .size(11)
                    .font(ui_font_semi)
                    .color(accent),
            )
            .padding([2, 8])
            .style(badge_style(soft_bg, accent, p.border_subtle)),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
        hairline_divider(mode),
    ]
    .spacing(8)
    .width(Length::Fill);

    if tasks.is_empty() {
        col = col.push(aligned_text(
            "None in this state",
            11.5,
            ui_font,
            Some(p.text_muted),
        ));
    } else {
        for task in tasks {
            let mut inner = column![
                row![
                    text(task.stage.short_label())
                        .size(10.5)
                        .font(Font::MONOSPACE)
                        .color(accent),
                    Space::new().width(Length::Fill),
                    text(format!("{}%", task.progress_percent))
                        .size(10.5)
                        .font(Font::MONOSPACE)
                        .color(p.text_secondary),
                ]
                .align_y(Alignment::Center),
                aligned_text(task.title.clone(), 12.0, ui_font_semi, Some(p.text_primary)),
                aligned_text(
                    task.status_message.clone(),
                    11.0,
                    ui_font,
                    Some(p.text_secondary),
                ),
            ]
            .spacing(4);

            if task.status == TaskState::InProgress {
                inner =
                    inner.push(progress_bar(0.0..=100.0, task.progress_percent as f32).girth(4));
            }
            if !task.artifact_refs.is_empty() {
                inner = inner.push(
                    text(task.artifact_refs.join(", "))
                        .size(10.5)
                        .font(Font::MONOSPACE)
                        .color(p.accent_text),
                );
            }

            col = col.push(
                container(inner)
                    .padding(10)
                    .width(Length::Fill)
                    .style(elevated_box_style(mode, 8.0)),
            );
        }
    }

    container(col)
        .padding(14)
        .width(Length::FillPortion(1))
        .style(card_style(mode, 10.0))
        .into()
}
