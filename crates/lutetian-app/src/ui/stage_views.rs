use crate::ui::blueprint_canvas::view_blueprint_canvas;
use crate::ui::dashboard::{DashboardMsg, DashboardState, page_header};
use crate::ui::icons::{UiIcon, cached_image_handle, ui_icon};
use crate::ui::theme::{
    ThemePalette, aligned_text, badge_style, card_style, danger_button_style, elevated_box_style,
    ghost_button_style, hairline_divider, pill_button_style, primary_button_style,
    sleek_input_style, sleek_row, sleek_scrollable,
};
use iced::widget::text::Wrapping;
use iced::widget::{Space, button, column, container, image, row, text, text_input};
use iced::{Alignment, ContentFit, Element, Font, Length};
use lutetian_core::{
    AngleApprovalStatus, BlueprintLayer, LutetianProject, PipelineStage, UiThemeMode,
    build_bill_of_materials,
};

// ============================================================================
// STAGE 1: INGESTION & SCALE ANCHORS (PROJECT BIBLE)
// ============================================================================

#[allow(clippy::too_many_arguments)]
pub fn view_stage1_tab<'a>(
    state: &'a DashboardState,
    project: &'a LutetianProject,
    mode: UiThemeMode,
    p: ThemePalette,
    ui_font: Font,
    ui_font_semi: Font,
    ui_font_bold: Font,
) -> Element<'a, DashboardMsg> {
    let locked = project.is_stage_locked(PipelineStage::Stage1Ingestion);

    let header_action: Element<'a, DashboardMsg> = if locked {
        row![
            stage_lock_badge(true, p, ui_font_semi),
            button(
                row![
                    ui_icon(UiIcon::LockOpen, 14.0, p.text_primary),
                    text("Unlock Stage 1").size(12).font(ui_font_semi),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .on_press(DashboardMsg::UnlockStage(PipelineStage::Stage1Ingestion))
            .padding([7, 13])
            .style(pill_button_style(mode)),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
    } else {
        row![
            stage_lock_badge(false, p, ui_font_semi),
            button(
                row![
                    ui_icon(UiIcon::Lock, 14.0, iced::Color::WHITE),
                    text("Approve & Lock Stage 1")
                        .size(12)
                        .font(ui_font_semi)
                        .color(iced::Color::WHITE),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .on_press(DashboardMsg::ApproveAndLockStage1)
            .padding([8, 14])
            .style(primary_button_style(mode)),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
    };

    let header = page_header(
        "Stage 1 · Project Bible & Metric Scale Anchors",
        Some("Lock reference style, lighting, and human/door/floor/road metric anchors before multi-view generation.".to_string()),
        Some(header_action),
        p,
        ui_font,
        ui_font_bold,
    );

    let bible = &project.bible;
    let bible_card = container(column![
        editable_text_row(
            mode,
            p,
            "Project Title",
            Some("Environment name persisted in Project Bible & Stage 6 Engine Sync".to_string()),
            &bible.title,
            "e.g. Courtyard Market District",
            !locked,
            DashboardMsg::BibleTitleChanged,
            ui_font,
            ui_font_semi,
        ),
        hairline_divider(mode),
        editable_text_row(
            mode,
            p,
            "Art Style",
            Some("Surface language, rendering target, material era".to_string()),
            &bible.art_style,
            "e.g. Stylized European PBR Realism",
            !locked,
            DashboardMsg::BibleArtStyleChanged,
            ui_font,
            ui_font_semi,
        ),
        hairline_divider(mode),
        editable_text_row(
            mode,
            p,
            "Mood & Atmosphere",
            None,
            &bible.mood,
            "e.g. Warm golden late-afternoon",
            !locked,
            DashboardMsg::BibleMoodChanged,
            ui_font,
            ui_font_semi,
        ),
        hairline_divider(mode),
        editable_text_row(
            mode,
            p,
            "Key Lighting & Palette",
            None,
            &bible.lighting,
            "e.g. Sun 38 deg elevation, warm limestone bounce",
            !locked,
            DashboardMsg::BibleLightingChanged,
            ui_font,
            ui_font_semi,
        ),
        hairline_divider(mode),
        editable_text_row(
            mode,
            p,
            "Material Palette Notes",
            None,
            &bible.palette_notes,
            "e.g. Ashlar stone, weathered oak, slate roof",
            !locked,
            DashboardMsg::BiblePaletteChanged,
            ui_font,
            ui_font_semi,
        ),
    ])
    .width(Length::Fill)
    .style(card_style(mode, 10.0));

    let anchors_card = container(column![
        container(
            row![
                ui_icon(UiIcon::Straighten, 16.0, p.accent_text),
                text("Metric Scale Anchors (meters)")
                    .size(13.5)
                    .font(ui_font_bold)
                    .color(p.text_primary),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([12, 18]),
        hairline_divider(mode),
        metric_row(
            mode,
            "Human Reference Height (m)",
            "Default 1.80m — enforces building >= 1.8x & door > human height",
            &state.anchor_human_buf,
            !locked,
            DashboardMsg::AnchorHumanChanged,
            ui_font,
            ui_font_semi,
        ),
        hairline_divider(mode),
        metric_row(
            mode,
            "Standard Door Height (m)",
            "Default 2.10m — must exceed human_height_m",
            &state.anchor_door_h_buf,
            !locked,
            DashboardMsg::AnchorDoorHeightChanged,
            ui_font,
            ui_font_semi,
        ),
        hairline_divider(mode),
        metric_row(
            mode,
            "Minimum Door Width (m)",
            "Default 1.00m (validator hard floor 0.85m)",
            &state.anchor_door_w_buf,
            !locked,
            DashboardMsg::AnchorDoorWidthChanged,
            ui_font,
            ui_font_semi,
        ),
        hairline_divider(mode),
        metric_row(
            mode,
            "Floor-to-Floor Height (m)",
            "Default 3.20m — governs multi-story building Z height",
            &state.anchor_floor_h_buf,
            !locked,
            DashboardMsg::AnchorFloorHeightChanged,
            ui_font,
            ui_font_semi,
        ),
        hairline_divider(mode),
        metric_row(
            mode,
            "Road Lane Width (m)",
            "Default 3.50m — minimum width for vehicular corridors",
            &state.anchor_road_w_buf,
            !locked,
            DashboardMsg::AnchorRoadLaneChanged,
            ui_font,
            ui_font_semi,
        ),
    ])
    .width(Length::Fill)
    .style(card_style(mode, 10.0));

    // Reference Images Gallery
    let mut ref_rows = column![].spacing(8);
    if bible.reference_images.is_empty() {
        ref_rows = ref_rows.push(aligned_text(
            "No reference images attached yet. Add a reference image or let Antigravity submit Stage 1 ingestion.",
            12.0,
            ui_font,
            Some(p.text_secondary),
        ));
    } else {
        for (idx, path) in bible.reference_images.iter().enumerate() {
            let thumb: Element<'a, DashboardMsg> = if let Some(handle) = cached_image_handle(path) {
                image(handle)
                    .width(Length::Fixed(56.0))
                    .height(Length::Fixed(42.0))
                    .content_fit(ContentFit::Cover)
                    .into()
            } else {
                container(ui_icon(UiIcon::PhotoLibrary, 18.0, p.text_muted))
                    .width(Length::Fixed(56.0))
                    .height(Length::Fixed(42.0))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .style(elevated_box_style(mode, 6.0))
                    .into()
            };

            let mut row_items: Vec<Element<'a, DashboardMsg>> = vec![
                thumb,
                aligned_text(path.clone(), 12.0, Font::MONOSPACE, Some(p.text_primary)),
            ];
            if !locked {
                row_items.push(
                    button(ui_icon(UiIcon::Delete, 14.0, p.danger))
                        .on_press(DashboardMsg::RemoveReferenceImage(idx))
                        .padding([6, 8])
                        .style(danger_button_style(mode))
                        .into(),
                );
            }
            ref_rows = ref_rows.push(
                container(
                    row(row_items)
                        .spacing(12)
                        .align_y(Alignment::Center)
                        .width(Length::Fill),
                )
                .padding([8, 12])
                .style(elevated_box_style(mode, 8.0)),
            );
        }
    }

    let add_ref_btn = button(
        row![
            ui_icon(UiIcon::Add, 14.0, p.text_primary),
            text("Attach Reference Image...")
                .size(12)
                .font(ui_font_semi),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .on_press_maybe(if locked {
        None
    } else {
        Some(DashboardMsg::AddReferenceImage)
    })
    .padding([7, 12])
    .style(pill_button_style(mode));

    let refs_card = container(
        column![
            row![
                ui_icon(UiIcon::PhotoLibrary, 16.0, p.accent_text),
                text("Reference Images")
                    .size(13.5)
                    .font(ui_font_bold)
                    .color(p.text_primary),
                Space::new().width(Length::Fill),
                add_ref_btn,
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            ref_rows,
        ]
        .spacing(12),
    )
    .padding([14, 18])
    .width(Length::Fill)
    .style(card_style(mode, 10.0));

    sleek_scrollable(
        column![header, bible_card, anchors_card, refs_card]
            .spacing(16)
            .width(Length::Fill),
        mode,
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

// ============================================================================
// STAGE 2: MULTI-VIEW COMPLETION GALLERY
// ============================================================================

#[allow(clippy::too_many_arguments)]
pub fn view_multiview_tab<'a>(
    state: &'a DashboardState,
    project: &'a LutetianProject,
    mode: UiThemeMode,
    p: ThemePalette,
    ui_font: Font,
    ui_font_semi: Font,
    ui_font_bold: Font,
) -> Element<'a, DashboardMsg> {
    let stage1_locked = project.is_stage_locked(PipelineStage::Stage1Ingestion);
    let stage2_locked = project.is_stage_locked(PipelineStage::Stage2MultiView);
    let approved_cnt = project
        .multi_view
        .slots
        .iter()
        .filter(|s| s.status == AngleApprovalStatus::Approved)
        .count();

    let right_badge: Element<'a, DashboardMsg> = row![
        container(
            text(format!("{approved_cnt}/5 Approved"))
                .size(12)
                .font(ui_font_semi)
                .color(if approved_cnt == 5 {
                    p.success
                } else {
                    p.accent_text
                }),
        )
        .padding([6, 12])
        .style(badge_style(
            if approved_cnt == 5 {
                p.success_soft
            } else {
                p.accent_soft
            },
            if approved_cnt == 5 {
                p.success
            } else {
                p.accent_text
            },
            p.border_subtle,
        )),
        if stage2_locked {
            Element::from(
                button(
                    row![
                        ui_icon(UiIcon::LockOpen, 14.0, p.text_primary),
                        text("Unlock Stage 2").size(12).font(ui_font_semi),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .on_press(DashboardMsg::UnlockStage(PipelineStage::Stage2MultiView))
                .padding([7, 13])
                .style(pill_button_style(mode)),
            )
        } else {
            Space::new().width(0).into()
        }
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into();

    let header = page_header(
        "Stage 2 · Multi-View Completion & Per-Angle Approval",
        Some("Inspect and approve Front, Back, Left, Right, and Bird's-Eye views before drafting the Master Blueprint.".to_string()),
        Some(right_badge),
        p,
        ui_font,
        ui_font_bold,
    );

    let mut content = column![header].spacing(14).width(Length::Fill);

    if !stage1_locked {
        content = content.push(
            container(
                row![
                    ui_icon(UiIcon::Warning, 16.0, p.warning),
                    aligned_text(
                        "Stage 1 Project Bible & Scale Anchors must be approved and locked before agents can submit Stage 2 angles.",
                        12.5,
                        ui_font_semi,
                        Some(p.warning),
                    ),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            )
            .padding([12, 16])
            .width(Length::Fill)
            .style(elevated_box_style(mode, 10.0)),
        );
    }

    for slot in &project.multi_view.slots {
        let angle = slot.angle;
        let note_val = state
            .angle_note_bufs
            .get(&angle)
            .map(String::as_str)
            .unwrap_or(&slot.notes);

        let (st_bg, st_fg) = match slot.status {
            AngleApprovalStatus::Approved => (p.success_soft, p.success),
            AngleApprovalStatus::Submitted => (p.accent_soft, p.accent_text),
            AngleApprovalStatus::RegenerateRequested => (p.danger_soft, p.danger),
            AngleApprovalStatus::Pending => (p.warning_soft, p.warning),
        };

        let preview: Element<'a, DashboardMsg> = if let Some(path) = &slot.image_path
            && let Some(handle) = cached_image_handle(path)
        {
            image(handle)
                .width(Length::Fixed(180.0))
                .height(Length::Fixed(120.0))
                .content_fit(ContentFit::Contain)
                .into()
        } else {
            container(
                column![
                    ui_icon(UiIcon::PhotoLibrary, 26.0, p.text_muted),
                    text(slot.image_path.as_deref().unwrap_or("No image attached"))
                        .size(11)
                        .font(Font::MONOSPACE)
                        .color(p.text_muted),
                ]
                .spacing(6)
                .align_x(Alignment::Center),
            )
            .width(Length::Fixed(180.0))
            .height(Length::Fixed(120.0))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(elevated_box_style(mode, 8.0))
            .into()
        };

        let attach_btn = button(
            row![
                ui_icon(UiIcon::FolderOpen, 14.0, p.text_primary),
                text("Attach Image...").size(11.5).font(ui_font_semi),
            ]
            .spacing(5)
            .align_y(Alignment::Center),
        )
        .on_press_maybe(if stage1_locked && !stage2_locked {
            Some(DashboardMsg::AttachAngleImage(angle))
        } else {
            None
        })
        .padding([6, 10])
        .style(pill_button_style(mode));

        let approve_btn = button(
            row![
                ui_icon(UiIcon::CheckCircle, 14.0, iced::Color::WHITE),
                text(if slot.status == AngleApprovalStatus::Approved {
                    "Approved"
                } else {
                    "Approve View"
                })
                .size(11.5)
                .font(ui_font_semi)
                .color(iced::Color::WHITE),
            ]
            .spacing(5)
            .align_y(Alignment::Center),
        )
        .on_press_maybe(if stage1_locked {
            Some(DashboardMsg::ApproveAngle(angle))
        } else {
            None
        })
        .padding([6, 12])
        .style(primary_button_style(mode));

        let regen_btn = button(
            row![
                ui_icon(UiIcon::Refresh, 14.0, p.danger),
                text("Request Regen")
                    .size(11.5)
                    .font(ui_font_semi)
                    .color(p.danger),
            ]
            .spacing(5)
            .align_y(Alignment::Center),
        )
        .on_press(DashboardMsg::RequestAngleRegen(angle))
        .padding([6, 10])
        .style(danger_button_style(mode));

        let info_col = column![
            row![
                text(angle.label())
                    .size(14.5)
                    .font(ui_font_bold)
                    .color(p.text_primary),
                container(
                    text(format!("{} · rev {}", slot.status.label(), slot.revision))
                        .size(11)
                        .font(ui_font_semi)
                        .color(st_fg),
                )
                .padding([3, 8])
                .style(badge_style(st_bg, st_fg, p.border_subtle)),
                Space::new().width(Length::Fill),
                attach_btn,
                approve_btn,
                regen_btn,
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            text(
                slot.image_path
                    .as_deref()
                    .unwrap_or("Awaiting image file path")
            )
            .size(11.5)
            .font(Font::MONOSPACE)
            .color(p.text_secondary),
            text_input("Notes or regeneration feedback...", note_val)
                .on_input(move |val| DashboardMsg::AngleNoteChanged(angle, val))
                .size(12.5)
                .padding([7, 10])
                .font(ui_font)
                .style(sleek_input_style(mode)),
        ]
        .spacing(8)
        .width(Length::Fill);

        content = content.push(
            container(
                row![preview, info_col]
                    .spacing(16)
                    .align_y(Alignment::Center),
            )
            .padding(14)
            .width(Length::Fill)
            .style(card_style(mode, 10.0)),
        );
    }

    sleek_scrollable(content, mode)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// ============================================================================
// STAGE 3: MASTER BLUEPRINT CAD VIEWER & ENTITY INSPECTOR
// ============================================================================

#[allow(clippy::too_many_arguments)]
pub fn view_blueprint_tab<'a>(
    state: &'a DashboardState,
    project: &'a LutetianProject,
    mode: UiThemeMode,
    p: ThemePalette,
    ui_font: Font,
    ui_font_semi: Font,
    ui_font_bold: Font,
) -> Element<'a, DashboardMsg> {
    let locked = project.is_stage_locked(PipelineStage::Stage3Blueprint);
    let passed = project.last_validation.passed;

    let top_actions: Element<'a, DashboardMsg> = row![
        button(
            row![
                ui_icon(
                    if passed {
                        UiIcon::VerifiedUser
                    } else {
                        UiIcon::Warning
                    },
                    14.0,
                    if passed { p.success } else { p.danger },
                ),
                text(if passed {
                    "Scale Valid (0 Errors)"
                } else {
                    "Scale Violated"
                })
                .size(12)
                .font(ui_font_semi)
                .color(if passed { p.success } else { p.danger }),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .on_press(DashboardMsg::RunScaleValidation)
        .padding([6, 12])
        .style(pill_button_style(mode)),
        if locked {
            Element::from(
                button(
                    row![
                        ui_icon(UiIcon::LockOpen, 14.0, p.text_primary),
                        text("Unlock Stage 3").size(12).font(ui_font_semi),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .on_press(DashboardMsg::UnlockStage(PipelineStage::Stage3Blueprint))
                .padding([6, 12])
                .style(pill_button_style(mode)),
            )
        } else {
            Element::from(
                button(
                    row![
                        ui_icon(UiIcon::Lock, 14.0, iced::Color::WHITE),
                        text("Lock Blueprint")
                            .size(12)
                            .font(ui_font_semi)
                            .color(iced::Color::WHITE),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .on_press(DashboardMsg::ApproveAndLockStage3)
                .padding([6, 12])
                .style(primary_button_style(mode)),
            )
        }
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into();

    let header = page_header(
        "Stage 3 · Master Blueprint CAD (Metric Grid)",
        Some(format!(
            "{:.1}m × {:.1}m · Pan (drag), Zoom (wheel), Click entities to inspect/edit dimensions, Drag 1.8m Human / 4.5m Vehicle markers",
            project.blueprint.world_width_m, project.blueprint.world_depth_m
        )),
        Some(top_actions),
        p,
        ui_font,
        ui_font_bold,
    );

    // Layer Toggle Strip + Zoom controls
    let mut layer_row = row![].spacing(6).align_y(Alignment::Center);
    for &layer in BlueprintLayer::all() {
        let active = state.canvas_state.visible_layers.contains(&layer);
        layer_row = layer_row.push(
            button(
                text(layer.label())
                    .size(11.5)
                    .font(ui_font_semi)
                    .color(if active { p.accent_text } else { p.text_muted })
                    .wrapping(Wrapping::None),
            )
            .on_press(DashboardMsg::ToggleBlueprintLayer(layer))
            .padding([5, 10])
            .style(ghost_button_style(mode, active)),
        );
    }

    let zoom_controls = row![
        button(ui_icon(UiIcon::ZoomOut, 15.0, p.text_primary))
            .on_press(DashboardMsg::CanvasZoomOut)
            .padding([5, 8])
            .style(pill_button_style(mode)),
        button(ui_icon(UiIcon::FitScreen, 15.0, p.text_primary))
            .on_press(DashboardMsg::CanvasFitWorld)
            .padding([5, 8])
            .style(pill_button_style(mode)),
        button(ui_icon(UiIcon::ZoomIn, 15.0, p.text_primary))
            .on_press(DashboardMsg::CanvasZoomIn)
            .padding([5, 8])
            .style(pill_button_style(mode)),
    ]
    .spacing(4)
    .align_y(Alignment::Center);

    let toolbar = container(
        row![layer_row, Space::new().width(Length::Fill), zoom_controls]
            .spacing(10)
            .align_y(Alignment::Center)
            .width(Length::Fill),
    )
    .padding([8, 12])
    .width(Length::Fill)
    .style(card_style(mode, 10.0));

    let canvas_card = container(view_blueprint_canvas(
        project,
        &state.canvas_state,
        mode,
        DashboardMsg::Canvas,
    ))
    .width(Length::FillPortion(3))
    .height(Length::Fill)
    .padding(1)
    .style(card_style(mode, 10.0));

    let inspector =
        view_blueprint_inspector(state, project, mode, p, ui_font, ui_font_semi, ui_font_bold);

    let split = row![canvas_card, inspector]
        .spacing(12)
        .width(Length::Fill)
        .height(Length::Fill);

    column![header, toolbar, split]
        .spacing(12)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn view_blueprint_inspector<'a>(
    state: &'a DashboardState,
    project: &'a LutetianProject,
    mode: UiThemeMode,
    p: ThemePalette,
    ui_font: Font,
    ui_font_semi: Font,
    ui_font_bold: Font,
) -> Element<'a, DashboardMsg> {
    let sel_id = state.canvas_state.selected_entity_id.as_deref();
    let mut body = column![
        row![
            ui_icon(UiIcon::Architecture, 15.0, p.accent_text),
            text("Entity Inspector")
                .size(13.5)
                .font(ui_font_bold)
                .color(p.text_primary),
        ]
        .spacing(7)
        .align_y(Alignment::Center),
        hairline_divider(mode),
    ]
    .spacing(10)
    .width(Length::Fill);

    if let Some(id) = sel_id {
        if let Some(fp) = project.blueprint.footprints.iter().find(|f| f.id == id) {
            body = body
                .push(text(&fp.id).size(13).font(Font::MONOSPACE).color(p.accent_text))
                .push(text(&fp.label).size(13).font(ui_font_bold).color(p.text_primary))
                .push(
                    text(format!("Asset ID: {}", fp.asset_id))
                        .size(11.5)
                        .font(Font::MONOSPACE)
                        .color(p.text_secondary),
                )
                .push(
                    text(format!(
                        "Pos: ({:.2}m, {:.2}m) · Elev: +{:.2}m · Floors: {}",
                        fp.rect.x_m, fp.rect.y_m, fp.base_elevation_m, fp.floor_count
                    ))
                    .size(11.5)
                    .font(Font::MONOSPACE)
                    .color(p.text_secondary),
                )
                .push(hairline_divider(mode))
                .push(inspector_dim_field(
                    "Width X (m)",
                    &state.selected_w_buf,
                    DashboardMsg::SelectedWidthChanged,
                    mode,
                    p,
                    ui_font_semi,
                ))
                .push(inspector_dim_field(
                    "Depth Y (m)",
                    &state.selected_d_buf,
                    DashboardMsg::SelectedDepthChanged,
                    mode,
                    p,
                    ui_font_semi,
                ))
                .push(inspector_dim_field(
                    "Height Z (m)",
                    &state.selected_h_buf,
                    DashboardMsg::SelectedHeightChanged,
                    mode,
                    p,
                    ui_font_semi,
                ))
                .push(aligned_text(
                    "Tip: Lower Height Z below 3.24m (e.g. 1.50) to watch the deterministic validator flag SCALE-002 live on the blueprint.",
                    11.0,
                    ui_font,
                    Some(p.text_muted),
                ));
        } else if let Some(door) = project.blueprint.doors.iter().find(|d| d.id == id) {
            body = body
                .push(
                    text(&door.id)
                        .size(13)
                        .font(Font::MONOSPACE)
                        .color(p.accent_text),
                )
                .push(
                    text(&door.label)
                        .size(13)
                        .font(ui_font_bold)
                        .color(p.text_primary),
                )
                .push(
                    text(format!("Parent: {}", door.parent_entity_id))
                        .size(11.5)
                        .font(Font::MONOSPACE)
                        .color(p.text_secondary),
                )
                .push(inspector_dim_field(
                    "Door Width (m)",
                    &state.selected_w_buf,
                    DashboardMsg::SelectedWidthChanged,
                    mode,
                    p,
                    ui_font_semi,
                ))
                .push(inspector_dim_field(
                    "Door Height (m)",
                    &state.selected_h_buf,
                    DashboardMsg::SelectedHeightChanged,
                    mode,
                    p,
                    ui_font_semi,
                ));
        } else if let Some(marker) = project.blueprint.scale_markers.iter().find(|m| m.id == id) {
            body = body
                .push(text(&marker.id).size(13).font(Font::MONOSPACE).color(p.success))
                .push(text(&marker.label).size(13).font(ui_font_bold).color(p.text_primary))
                .push(
                    text(format!("Position: ({:.2}m, {:.2}m)", marker.x_m, marker.y_m))
                        .size(12)
                        .font(Font::MONOSPACE)
                        .color(p.text_secondary),
                )
                .push(aligned_text(
                    "Drag directly on the CAD canvas to measure clearance next to doorways, benches, and roads.",
                    11.5,
                    ui_font,
                    Some(p.text_secondary),
                ));
        } else if let Some(zone) = project.blueprint.zones.iter().find(|z| z.id == id) {
            body = body
                .push(
                    text(&zone.id)
                        .size(13)
                        .font(Font::MONOSPACE)
                        .color(p.accent_text),
                )
                .push(
                    text(&zone.label)
                        .size(13)
                        .font(ui_font_bold)
                        .color(p.text_primary),
                )
                .push(
                    text(format!(
                        "Surface: {} · Area: {:.1}m × {:.1}m",
                        zone.kind.label(),
                        zone.rect.width_m,
                        zone.rect.depth_m
                    ))
                    .size(11.5)
                    .font(Font::MONOSPACE)
                    .color(p.text_secondary),
                );
        }

        // Show entity violations if any
        let entity_violations: Vec<_> = project
            .last_validation
            .violations
            .iter()
            .filter(|v| v.entity_id == id)
            .collect();
        if !entity_violations.is_empty() {
            body = body.push(hairline_divider(mode));
            for v in entity_violations {
                body = body.push(
                    container(
                        column![
                            text(format!("{} VIOLATED", v.rule_id))
                                .size(11.5)
                                .font(ui_font_bold)
                                .color(p.danger),
                            text(&v.message)
                                .size(11.5)
                                .font(ui_font)
                                .color(p.text_primary),
                            text(format!("Expected: {} | Actual: {}", v.expected, v.actual))
                                .size(11.0)
                                .font(Font::MONOSPACE)
                                .color(p.text_secondary),
                        ]
                        .spacing(4),
                    )
                    .padding(10)
                    .width(Length::Fill)
                    .style(elevated_box_style(mode, 8.0)),
                );
            }
        }
    } else {
        body = body.push(aligned_text(
            "Click any footprint, doorway, zone, or scale marker on the 2D CAD grid to inspect or edit exact metric dimensions.",
            12.0,
            ui_font,
            Some(p.text_secondary),
        ));
        body = body.push(hairline_divider(mode));
        body = body.push(
            row![
                ui_icon(UiIcon::AccessibilityNew, 14.0, p.success),
                text(format!(
                    "Human Scale: {:.2}m",
                    project.bible.scale_anchors.human_height_m
                ))
                .size(11.5)
                .font(Font::MONOSPACE)
                .color(p.text_primary),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        );
        body = body.push(
            row![
                ui_icon(UiIcon::DirectionsCar, 14.0, p.warning),
                text(format!(
                    "Vehicle: {:.1}m × {:.1}m",
                    project.bible.scale_anchors.vehicle_length_m,
                    project.bible.scale_anchors.vehicle_width_m
                ))
                .size(11.5)
                .font(Font::MONOSPACE)
                .color(p.text_primary),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        );
        body = body.push(
            text(format!(
                "• Door: {:.2}m × {:.2}m\n• Floor: {:.2}m\n• Road Lane: {:.2}m",
                project.bible.scale_anchors.door_width_m,
                project.bible.scale_anchors.door_height_m,
                project.bible.scale_anchors.floor_height_m,
                project.bible.scale_anchors.road_lane_width_m,
            ))
            .size(11.5)
            .font(Font::MONOSPACE)
            .color(p.text_secondary),
        );
    }

    container(sleek_scrollable(body, mode).height(Length::Fill))
        .padding(14)
        .width(Length::Fixed(275.0))
        .height(Length::Fill)
        .style(card_style(mode, 10.0))
        .into()
}

fn inspector_dim_field<'a>(
    label: &'a str,
    val: &'a str,
    on_change: impl Fn(String) -> DashboardMsg + 'a,
    mode: UiThemeMode,
    p: ThemePalette,
    ui_font_semi: Font,
) -> Element<'a, DashboardMsg> {
    column![
        text(label)
            .size(11.5)
            .font(ui_font_semi)
            .color(p.text_secondary),
        text_input("0.00", val)
            .on_input(on_change)
            .size(12.5)
            .padding([6, 10])
            .font(Font::MONOSPACE)
            .style(sleek_input_style(mode)),
    ]
    .spacing(4)
    .width(Length::Fill)
    .into()
}

// ============================================================================
// STAGE 4 & 6: ASSET CATALOG (DEDUPLICATED BOM) & ENGINE SYNC EXPORT
// ============================================================================

pub fn view_assets_tab<'a>(
    project: &'a LutetianProject,
    mode: UiThemeMode,
    p: ThemePalette,
    ui_font: Font,
    ui_font_semi: Font,
    ui_font_bold: Font,
) -> Element<'a, DashboardMsg> {
    let bom = build_bill_of_materials(&project.assets, &project.blueprint);
    let total_instances: usize = bom.iter().map(|b| b.instance_count).sum();
    let stage4_locked = project.is_stage_locked(PipelineStage::Stage4Assets);

    let actions: Element<'a, DashboardMsg> = row![
        container(
            text(format!(
                "{} Unique Assets · {} Instances",
                bom.len(),
                total_instances
            ))
            .size(12)
            .font(ui_font_semi)
            .color(p.accent_text),
        )
        .padding([6, 12])
        .style(badge_style(p.accent_soft, p.accent_text, p.border_subtle)),
        if !stage4_locked {
            Element::from(
                button(
                    row![
                        ui_icon(UiIcon::Lock, 14.0, iced::Color::WHITE),
                        text("Lock Asset Catalog")
                            .size(12)
                            .font(ui_font_semi)
                            .color(iced::Color::WHITE),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .on_press(DashboardMsg::ApproveAndLockStage4)
                .padding([7, 13])
                .style(primary_button_style(mode)),
            )
        } else {
            Space::new().width(0).into()
        },
        button(
            row![
                ui_icon(UiIcon::Download, 14.0, p.text_primary),
                text("Export Engine Bundle (Stage 6)")
                    .size(12)
                    .font(ui_font_semi),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .on_press(DashboardMsg::ExportEngineBundle)
        .padding([7, 13])
        .style(pill_button_style(mode)),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into();

    let header = page_header(
        "Stage 4 & 6 · Deduplicated Asset Catalog & Engine Sync",
        Some("Each unique modular asset carries locked metric bounding boxes (W × D × H) and an isolated reference slot.".to_string()),
        Some(actions),
        p,
        ui_font,
        ui_font_bold,
    );

    let mut list = column![header].spacing(14).width(Length::Fill);

    if bom.is_empty() {
        list = list.push(
            container(aligned_text(
                "No unique AssetSpecs registered yet. Lock Stage 3 Master Blueprint and let Antigravity submit deduplicated assets via submit_asset_spec.",
                13.0,
                ui_font,
                Some(p.text_secondary),
            ))
            .padding(20)
            .width(Length::Fill)
            .style(card_style(mode, 10.0)),
        );
    } else {
        for entry in bom {
            let asset = entry.asset;
            let thumb: Element<'a, DashboardMsg> = if let Some(path) = &asset.isolated_image_path
                && let Some(handle) = cached_image_handle(path)
            {
                image(handle)
                    .width(Length::Fixed(104.0))
                    .height(Length::Fixed(80.0))
                    .content_fit(ContentFit::Contain)
                    .into()
            } else {
                container(
                    column![
                        ui_icon(UiIcon::ViewInAr, 22.0, p.text_muted),
                        text("Isolated Slot")
                            .size(10.5)
                            .font(ui_font)
                            .color(p.text_muted),
                    ]
                    .spacing(4)
                    .align_x(Alignment::Center),
                )
                .width(Length::Fixed(104.0))
                .height(Length::Fixed(80.0))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(elevated_box_style(mode, 8.0))
                .into()
            };

            let bbox_str = format!(
                "{:.2}m W × {:.2}m D × {:.2}m H",
                asset.bbox_m.width_m, asset.bbox_m.depth_m, asset.bbox_m.height_m
            );

            let asset_id_for_btn = asset.asset_id.clone();
            let card_body = row![
                thumb,
                column![
                    row![
                        text(asset.asset_id.clone())
                            .size(13)
                            .font(Font::MONOSPACE)
                            .color(p.accent_text),
                        text("·").size(12).color(p.text_muted),
                        text(asset.name.clone())
                            .size(14)
                            .font(ui_font_bold)
                            .color(p.text_primary),
                        container(
                            text(asset.category.label())
                                .size(11)
                                .font(ui_font_semi)
                                .color(p.text_secondary),
                        )
                        .padding([2, 8])
                        .style(badge_style(
                            p.bg_elevated,
                            p.text_secondary,
                            p.border_subtle
                        )),
                        Space::new().width(Length::Fill),
                        container(
                            text(format!("{}× placed", entry.instance_count))
                                .size(11.5)
                                .font(ui_font_semi)
                                .color(p.success),
                        )
                        .padding([3, 9])
                        .style(badge_style(
                            p.success_soft,
                            p.success,
                            p.border_subtle
                        )),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                    text(format!(
                        "Bounding Box: {bbox_str}   |   Pivot: {}   |   Refs: {}",
                        asset.pivot,
                        if entry.entity_ids.is_empty() {
                            "none".to_string()
                        } else {
                            entry.entity_ids.join(", ")
                        }
                    ))
                    .size(11.5)
                    .font(Font::MONOSPACE)
                    .color(p.text_secondary),
                    row![
                        aligned_text(
                            if asset.notes.is_empty() {
                                asset
                                    .isolated_image_path
                                    .clone()
                                    .unwrap_or_else(|| "No isolated image attached".to_string())
                            } else {
                                asset.notes.clone()
                            },
                            12.0,
                            ui_font,
                            Some(p.text_secondary),
                        ),
                        button(
                            row![
                                ui_icon(UiIcon::PhotoLibrary, 13.0, p.text_primary),
                                text("Attach Isolated Ref...").size(11.5).font(ui_font_semi),
                            ]
                            .spacing(5)
                            .align_y(Alignment::Center),
                        )
                        .on_press(DashboardMsg::AttachAssetImage(asset_id_for_btn))
                        .padding([5, 10])
                        .style(pill_button_style(mode)),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                ]
                .spacing(6)
                .width(Length::Fill),
            ]
            .spacing(14)
            .align_y(Alignment::Center);

            list = list.push(
                container(card_body)
                    .padding(14)
                    .width(Length::Fill)
                    .style(card_style(mode, 10.0)),
            );
        }
    }

    sleek_scrollable(list, mode)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// ============================================================================
// SHARED STAGE HELPERS
// ============================================================================

fn stage_lock_badge<'a>(
    locked: bool,
    p: ThemePalette,
    ui_font_semi: Font,
) -> Element<'a, DashboardMsg> {
    container(
        row![
            ui_icon(
                if locked {
                    UiIcon::Lock
                } else {
                    UiIcon::LockOpen
                },
                13.0,
                if locked { p.success } else { p.warning },
            ),
            text(if locked {
                "Approved & Locked"
            } else {
                "Unlocked / Editable"
            })
            .size(11.5)
            .font(ui_font_semi)
            .color(if locked { p.success } else { p.warning }),
        ]
        .spacing(5)
        .align_y(Alignment::Center),
    )
    .padding([5, 11])
    .style(badge_style(
        if locked {
            p.success_soft
        } else {
            p.warning_soft
        },
        if locked { p.success } else { p.warning },
        p.border_subtle,
    ))
    .into()
}

#[allow(clippy::too_many_arguments)]
fn editable_text_row<'a>(
    mode: UiThemeMode,
    _p: ThemePalette,
    title: &'a str,
    subtitle: Option<String>,
    value: &'a str,
    placeholder: &'a str,
    editable: bool,
    on_change: impl Fn(String) -> DashboardMsg + 'a,
    ui_font: Font,
    ui_font_semi: Font,
) -> Element<'a, DashboardMsg> {
    let mut input = text_input(placeholder, value)
        .size(12.5)
        .padding([7, 11])
        .width(Length::Fixed(340.0))
        .font(ui_font)
        .style(sleek_input_style(mode));
    if editable {
        input = input.on_input(on_change);
    }
    sleek_row(mode, title, subtitle, input.into(), ui_font, ui_font_semi)
}

#[allow(clippy::too_many_arguments)]
fn metric_row<'a>(
    mode: UiThemeMode,
    title: &'a str,
    subtitle: &'a str,
    value: &'a str,
    editable: bool,
    on_change: impl Fn(String) -> DashboardMsg + 'a,
    ui_font: Font,
    ui_font_semi: Font,
) -> Element<'a, DashboardMsg> {
    let mut input = text_input("1.80", value)
        .size(12.5)
        .padding([7, 11])
        .width(Length::Fixed(130.0))
        .font(Font::MONOSPACE)
        .style(sleek_input_style(mode));
    if editable {
        input = input.on_input(on_change);
    }
    sleek_row(
        mode,
        title,
        Some(subtitle.to_string()),
        input.into(),
        ui_font,
        ui_font_semi,
    )
}
