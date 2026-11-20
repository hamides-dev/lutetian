use crate::ui::theme::{ThemePalette, font_for_ui, palette_for};
use iced::alignment::{Horizontal, Vertical};
use iced::font::Weight;
use iced::widget::canvas::{
    self, Action, Event, Frame, Geometry, Path, Program, Stroke, Text as CanvasText,
};
use iced::{Color, Element, Font, Length, Point, Rectangle, Renderer, Size, Theme, Vector, mouse};
use lutetian_core::{
    BlueprintLayer, FootprintKind, LutetianProject, ScaleMarkerKind, UiThemeMode, ZoneSurfaceKind,
};
use std::collections::{BTreeSet, HashSet};

const RULER_LEFT: f32 = 38.0;
const RULER_TOP: f32 = 26.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CanvasToolMode {
    #[default]
    SelectPan,
    MoveEntity,
    MeasureRuler,
}

#[derive(Debug, Clone)]
pub struct BlueprintCanvasState {
    pub zoom_px_per_m: f32,
    pub pan_x_px: f32,
    pub pan_y_px: f32,
    pub visible_layers: BTreeSet<BlueprintLayer>,
    pub selected_entity_id: Option<String>,
    pub tool_mode: CanvasToolMode,
    pub cursor_world_m: Option<(f64, f64)>,
    pub measure_line: Option<((f64, f64), (f64, f64))>,
}

impl Default for BlueprintCanvasState {
    fn default() -> Self {
        let mut visible_layers = BTreeSet::new();
        for &layer in BlueprintLayer::all() {
            visible_layers.insert(layer);
        }
        Self {
            zoom_px_per_m: 16.0,
            pan_x_px: 26.0,
            pan_y_px: 24.0,
            visible_layers,
            selected_entity_id: None,
            tool_mode: CanvasToolMode::SelectPan,
            cursor_world_m: None,
            measure_line: None,
        }
    }
}

impl BlueprintCanvasState {
    pub fn fit_world(&mut self, world_w_m: f64, world_d_m: f64, view_w: f32, view_h: f32) {
        let avail_w = (view_w - RULER_LEFT - 48.0).max(160.0);
        let avail_h = (view_h - RULER_TOP - 48.0).max(160.0);
        let zx = avail_w / (world_w_m.max(4.0) as f32);
        let zy = avail_h / (world_d_m.max(4.0) as f32);
        let z = zx.min(zy).clamp(5.0, 64.0);
        self.zoom_px_per_m = z;
        let drawn_w = (world_w_m as f32) * z;
        let drawn_h = (world_d_m as f32) * z;
        self.pan_x_px = ((avail_w - drawn_w) * 0.5 + 24.0).max(12.0);
        self.pan_y_px = ((avail_h - drawn_h) * 0.5 + 24.0).max(12.0);
    }

    pub fn focus_world_point(&mut self, world_x_m: f64, world_y_m: f64, view_w: f32, view_h: f32) {
        let avail_w = (view_w - RULER_LEFT).max(200.0);
        let avail_h = (view_h - RULER_TOP).max(200.0);
        if self.zoom_px_per_m < 14.0 {
            self.zoom_px_per_m = 18.0;
        }
        self.pan_x_px = avail_w * 0.5 - (world_x_m as f32) * self.zoom_px_per_m;
        self.pan_y_px = avail_h * 0.5 - (world_y_m as f32) * self.zoom_px_per_m;
    }

    fn world_to_screen(&self, x_m: f64, y_m: f64) -> Point {
        Point::new(
            RULER_LEFT + self.pan_x_px + (x_m as f32) * self.zoom_px_per_m,
            RULER_TOP + self.pan_y_px + (y_m as f32) * self.zoom_px_per_m,
        )
    }

    fn screen_to_world(&self, p: Point) -> (f64, f64) {
        let wx = ((p.x - RULER_LEFT - self.pan_x_px) / self.zoom_px_per_m) as f64;
        let wy = ((p.y - RULER_TOP - self.pan_y_px) / self.zoom_px_per_m) as f64;
        (wx, wy)
    }
}

#[derive(Debug, Clone)]
pub enum CanvasMsg {
    PanBy(Vector),
    ZoomAt {
        delta_y: f32,
        cursor: Point,
    },
    SelectEntity(Option<String>),
    MoveScaleMarker {
        marker_id: String,
        x_m: f64,
        y_m: f64,
    },
    MoveEntityBy {
        entity_id: String,
        dx_m: f64,
        dy_m: f64,
    },
    UpdateCursorWorld(Option<(f64, f64)>),
    SetMeasureLine(Option<((f64, f64), (f64, f64))>),
}

#[derive(Debug, Default)]
pub struct CanvasInteractionState {
    dragging_pan: bool,
    dragging_marker_id: Option<String>,
    dragging_entity_id: Option<String>,
    measuring_start: Option<(f64, f64)>,
    last_cursor: Option<Point>,
    press_origin: Option<Point>,
}

pub struct BlueprintCadProgram<'a> {
    pub project: &'a LutetianProject,
    pub state: &'a BlueprintCanvasState,
    pub mode: UiThemeMode,
}

impl<'a> Program<CanvasMsg> for BlueprintCadProgram<'a> {
    type State = CanvasInteractionState;

    fn update(
        &self,
        interaction: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<CanvasMsg>> {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let local = cursor.position_in(bounds)?;
                if local.x <= RULER_LEFT || local.y <= RULER_TOP {
                    return None;
                }
                interaction.last_cursor = Some(local);
                interaction.press_origin = Some(local);

                let (wx, wy) = self.state.screen_to_world(local);
                if self.state.tool_mode == CanvasToolMode::MeasureRuler {
                    let snapped = snap_quarter(wx, wy);
                    interaction.measuring_start = Some(snapped);
                    return Some(
                        Action::publish(CanvasMsg::SetMeasureLine(Some((snapped, snapped))))
                            .and_capture(),
                    );
                }

                // Scale marker grab is always supported when layer visible
                if self
                    .state
                    .visible_layers
                    .contains(&BlueprintLayer::NavigationScale)
                {
                    for marker in &self.project.blueprint.scale_markers {
                        let mp = self.state.world_to_screen(marker.x_m, marker.y_m);
                        if (local.x - mp.x).hypot(local.y - mp.y) <= 16.0 {
                            interaction.dragging_marker_id = Some(marker.id.clone());
                            return Some(
                                Action::publish(CanvasMsg::SelectEntity(Some(marker.id.clone())))
                                    .and_capture(),
                            );
                        }
                    }
                }

                if self.state.tool_mode == CanvasToolMode::MoveEntity
                    && let Some(picked) = hit_test_movable(self.project, wx, wy)
                {
                    interaction.dragging_entity_id = Some(picked.clone());
                    return Some(
                        Action::publish(CanvasMsg::SelectEntity(Some(picked))).and_capture(),
                    );
                }

                interaction.dragging_pan = true;
                None
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle))
            | Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                let local = cursor.position_in(bounds)?;
                interaction.last_cursor = Some(local);
                interaction.dragging_pan = true;
                Some(Action::capture())
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let Some(local) = cursor.position_in(bounds) else {
                    if self.state.cursor_world_m.is_some() {
                        return Some(Action::publish(CanvasMsg::UpdateCursorWorld(None)));
                    }
                    return None;
                };
                let prev = interaction.last_cursor.replace(local);
                let (wx, wy) = self.state.screen_to_world(local);

                if let Some(start) = interaction.measuring_start {
                    let end = snap_quarter(wx, wy);
                    return Some(
                        Action::publish(CanvasMsg::SetMeasureLine(Some((start, end))))
                            .and_capture(),
                    );
                }

                if let Some(marker_id) = interaction.dragging_marker_id.clone() {
                    let clamped_x = wx.clamp(0.0, self.project.blueprint.world_width_m);
                    let clamped_y = wy.clamp(0.0, self.project.blueprint.world_depth_m);
                    return Some(
                        Action::publish(CanvasMsg::MoveScaleMarker {
                            marker_id,
                            x_m: (clamped_x * 20.0).round() / 20.0,
                            y_m: (clamped_y * 20.0).round() / 20.0,
                        })
                        .and_capture(),
                    );
                }

                if let Some(entity_id) = interaction.dragging_entity_id.clone()
                    && let Some(p) = prev
                {
                    let dx_m = ((local.x - p.x) / self.state.zoom_px_per_m) as f64;
                    let dy_m = ((local.y - p.y) / self.state.zoom_px_per_m) as f64;
                    if dx_m.abs() > 0.001 || dy_m.abs() > 0.001 {
                        return Some(
                            Action::publish(CanvasMsg::MoveEntityBy {
                                entity_id,
                                dx_m,
                                dy_m,
                            })
                            .and_capture(),
                        );
                    }
                }

                if interaction.dragging_pan
                    && let Some(p) = prev
                {
                    let delta = Vector::new(local.x - p.x, local.y - p.y);
                    if delta.x.abs() > 0.1 || delta.y.abs() > 0.1 {
                        return Some(Action::publish(CanvasMsg::PanBy(delta)).and_capture());
                    }
                }

                if local.x > RULER_LEFT && local.y > RULER_TOP {
                    let snapped = snap_quarter(wx, wy);
                    if self.state.cursor_world_m != Some(snapped) {
                        return Some(Action::publish(CanvasMsg::UpdateCursorWorld(Some(snapped))));
                    }
                }
                None
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let was_special = interaction.dragging_marker_id.take().is_some()
                    || interaction.dragging_entity_id.take().is_some()
                    || interaction.measuring_start.take().is_some();
                interaction.dragging_pan = false;
                let press_origin = interaction.press_origin.take();
                interaction.last_cursor = None;

                if was_special {
                    return Some(Action::capture());
                }

                if let (Some(orig), Some(local)) = (press_origin, cursor.position_in(bounds)) {
                    let dist = ((local.x - orig.x).powi(2) + (local.y - orig.y).powi(2)).sqrt();
                    if dist < 5.0 && local.x > RULER_LEFT && local.y > RULER_TOP {
                        let (wx, wy) = self.state.screen_to_world(local);
                        let picked = hit_test_entity(self.project, wx, wy);
                        return Some(
                            Action::publish(CanvasMsg::SelectEntity(picked)).and_capture(),
                        );
                    }
                }
                None
            }
            Event::Mouse(mouse::Event::ButtonReleased(_)) => {
                interaction.dragging_pan = false;
                interaction.dragging_marker_id = None;
                interaction.dragging_entity_id = None;
                interaction.measuring_start = None;
                interaction.last_cursor = None;
                None
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let local = cursor.position_in(bounds)?;
                let dy = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y * 1.6,
                    mouse::ScrollDelta::Pixels { y, .. } => *y / 28.0,
                };
                if dy.abs() > 0.001 {
                    Some(
                        Action::publish(CanvasMsg::ZoomAt {
                            delta_y: dy,
                            cursor: local,
                        })
                        .and_capture(),
                    )
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let p = palette_for(self.mode);
        let bp = &self.project.blueprint;
        let z = self.state.zoom_px_per_m;
        let ui_font = font_for_ui(Weight::Medium);
        let mono_font = Font::MONOSPACE;

        let violated_ids: HashSet<&str> = self
            .project
            .last_validation
            .violations
            .iter()
            .map(|v| v.entity_id.as_str())
            .collect();

        // 1. Canvas background
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), p.bg_input);

        // 2. Zoning & Surface Layer
        if self
            .state
            .visible_layers
            .contains(&BlueprintLayer::ZoningSurface)
        {
            for zone in &bp.zones {
                let fill = zone_color(zone.kind, self.mode);
                let top_left = self.state.world_to_screen(zone.rect.x_m, zone.rect.y_m);
                let size = Size::new(
                    (zone.rect.width_m as f32) * z,
                    (zone.rect.depth_m as f32) * z,
                );
                frame.fill_rectangle(top_left, size, fill);
                frame.stroke(
                    &Path::rectangle(top_left, size),
                    Stroke {
                        width: 1.0,
                        style: canvas::Style::Solid(Color::from_rgba(fill.r, fill.g, fill.b, 0.45)),
                        ..Stroke::default()
                    },
                );

                if z >= 8.0 {
                    let center = zone.rect.center();
                    let cp = self.state.world_to_screen(center.x_m, center.y_m);
                    frame.fill_text(CanvasText {
                        content: zone.label.clone(),
                        position: cp,
                        color: Color::from_rgba(
                            p.text_secondary.r,
                            p.text_secondary.g,
                            p.text_secondary.b,
                            0.72,
                        ),
                        size: 10.5.into(),
                        font: ui_font,
                        align_x: Horizontal::Center.into(),
                        align_y: Vertical::Center,
                        ..CanvasText::default()
                    });
                }
            }
        }

        // 3. Metric Grid & World Bounds
        draw_metric_grid(
            &mut frame,
            bounds.size(),
            self.state,
            bp.world_width_m,
            bp.world_depth_m,
            bp.grid_unit_m,
            p,
        );

        // 4. Elevation & Height Layer
        let show_elev = self
            .state
            .visible_layers
            .contains(&BlueprintLayer::ElevationHeight);
        if show_elev {
            for elev in &bp.elevations {
                let tl = self.state.world_to_screen(elev.rect.x_m, elev.rect.y_m);
                let sz = Size::new(
                    (elev.rect.width_m as f32) * z,
                    (elev.rect.depth_m as f32) * z,
                );
                let contour_col = if self.mode == UiThemeMode::Dark {
                    Color::from_rgba(0.788, 0.722, 0.831, 0.38)
                } else {
                    Color::from_rgba(0.408, 0.361, 0.455, 0.38)
                };
                frame.stroke(
                    &Path::rectangle(tl, sz),
                    Stroke {
                        width: 1.3,
                        style: canvas::Style::Solid(contour_col),
                        line_dash: canvas::LineDash {
                            segments: &[6.0, 4.0],
                            offset: 0,
                        },
                        ..Stroke::default()
                    },
                );
                frame.fill_text(CanvasText {
                    content: format!("+{:.2}m EL", elev.elevation_m),
                    position: Point::new(tl.x + 6.0, tl.y + 6.0),
                    color: p.accent_text,
                    size: 10.0.into(),
                    font: mono_font,
                    align_x: Horizontal::Left.into(),
                    align_y: Vertical::Top,
                    ..CanvasText::default()
                });
            }
        }

        // 5. Navigation & Scale Corridors
        if self
            .state
            .visible_layers
            .contains(&BlueprintLayer::NavigationScale)
        {
            for nav in &bp.navigation_paths {
                if nav.points_m.len() < 2 {
                    continue;
                }
                let is_violated = violated_ids.contains(nav.id.as_str());
                let corridor_col = if is_violated {
                    Color::from_rgba(p.danger.r, p.danger.g, p.danger.b, 0.32)
                } else if nav.is_vehicular {
                    Color::from_rgba(0.965, 0.714, 0.282, 0.20)
                } else {
                    Color::from_rgba(0.243, 0.804, 0.588, 0.20)
                };
                let center_col = if is_violated {
                    p.danger
                } else if nav.is_vehicular {
                    p.warning
                } else {
                    p.success
                };

                let path = Path::new(|builder| {
                    let first = self
                        .state
                        .world_to_screen(nav.points_m[0].x_m, nav.points_m[0].y_m);
                    builder.move_to(first);
                    for pt in &nav.points_m[1..] {
                        builder.line_to(self.state.world_to_screen(pt.x_m, pt.y_m));
                    }
                });
                frame.stroke(
                    &path,
                    Stroke {
                        width: ((nav.width_m as f32) * z).max(4.0),
                        style: canvas::Style::Solid(corridor_col),
                        line_cap: canvas::LineCap::Round,
                        line_join: canvas::LineJoin::Round,
                        ..Stroke::default()
                    },
                );
                frame.stroke(
                    &path,
                    Stroke {
                        width: 1.5,
                        style: canvas::Style::Solid(center_col),
                        line_dash: canvas::LineDash {
                            segments: &[5.0, 4.0],
                            offset: 0,
                        },
                        ..Stroke::default()
                    },
                );

                let mid = nav.points_m[nav.points_m.len() / 2];
                let mp = self.state.world_to_screen(mid.x_m, mid.y_m);
                frame.fill_text(CanvasText {
                    content: format!("{} ({:.1}m)", nav.id, nav.width_m),
                    position: Point::new(mp.x, mp.y - 10.0),
                    color: center_col,
                    size: 9.5.into(),
                    font: mono_font,
                    align_x: Horizontal::Center.into(),
                    align_y: Vertical::Bottom,
                    ..CanvasText::default()
                });
            }
        }

        // 6. Footprint & Metrics Layer
        if self
            .state
            .visible_layers
            .contains(&BlueprintLayer::FootprintMetrics)
        {
            for fp in &bp.footprints {
                let is_selected = self
                    .state
                    .selected_entity_id
                    .as_deref()
                    .is_some_and(|s| s == fp.id);
                let is_violated = violated_ids.contains(fp.id.as_str());
                let (fill, stroke_base) = footprint_colors(fp.kind, self.mode, p);
                let stroke_col = if is_violated {
                    p.danger
                } else if is_selected {
                    p.accent_text
                } else {
                    stroke_base
                };

                let poly = rotated_rect_path(self.state, &fp.rect);
                frame.fill(&poly, fill);
                frame.stroke(
                    &poly,
                    Stroke {
                        width: if is_violated || is_selected { 2.6 } else { 1.4 },
                        style: canvas::Style::Solid(stroke_col),
                        ..Stroke::default()
                    },
                );

                if is_violated {
                    let (ax0, ay0, ax1, ay1) = fp.rect.world_aabb();
                    let p0 = self.state.world_to_screen(ax0, ay0);
                    let p1 = self.state.world_to_screen(ax1, ay1);
                    frame.stroke(
                        &Path::rectangle(
                            Point::new(p0.x - 4.0, p0.y - 4.0),
                            Size::new((p1.x - p0.x) + 8.0, (p1.y - p0.y) + 8.0),
                        ),
                        Stroke {
                            width: 1.5,
                            style: canvas::Style::Solid(p.danger),
                            line_dash: canvas::LineDash {
                                segments: &[4.0, 3.0],
                                offset: 0,
                            },
                            ..Stroke::default()
                        },
                    );
                }

                let c = fp.rect.center();
                let cp = self.state.world_to_screen(c.x_m, c.y_m);
                let pixel_w = (fp.rect.width_m as f32) * z;
                if pixel_w >= 26.0 {
                    let label_txt = if show_elev {
                        format!(
                            "{:.1}×{:.1}m\n+{:.1}→{:.1}m Z",
                            fp.rect.width_m,
                            fp.rect.depth_m,
                            fp.base_elevation_m,
                            fp.base_elevation_m + fp.height_m
                        )
                    } else {
                        format!(
                            "{:.1}×{:.1}m\nH {:.1}m",
                            fp.rect.width_m, fp.rect.depth_m, fp.height_m
                        )
                    };
                    frame.fill_text(CanvasText {
                        content: label_txt,
                        position: cp,
                        color: if is_violated {
                            p.danger
                        } else {
                            p.text_primary
                        },
                        size: 9.5.into(),
                        font: mono_font,
                        align_x: Horizontal::Center.into(),
                        align_y: Vertical::Center,
                        ..CanvasText::default()
                    });
                }
            }

            // Doors on footprints
            for door in &bp.doors {
                let dp = self
                    .state
                    .world_to_screen(door.position_m.x_m, door.position_m.y_m);
                let is_violated = violated_ids.contains(door.id.as_str());
                let is_selected = self
                    .state
                    .selected_entity_id
                    .as_deref()
                    .is_some_and(|s| s == door.id);
                let col = if is_violated {
                    p.danger
                } else if is_selected {
                    p.accent_text
                } else {
                    p.success
                };
                let r = ((door.width_m as f32) * z * 0.5).max(5.0);
                frame.fill(&Path::circle(dp, 3.5), col);
                frame.stroke(
                    &Path::circle(dp, r),
                    Stroke {
                        width: 1.5,
                        style: canvas::Style::Solid(col),
                        ..Stroke::default()
                    },
                );
                if z >= 10.0 {
                    frame.fill_text(CanvasText {
                        content: format!("D {:.2}×{:.2}m", door.width_m, door.height_m),
                        position: Point::new(dp.x, dp.y + r + 3.0),
                        color: col,
                        size: 9.0.into(),
                        font: mono_font,
                        align_x: Horizontal::Center.into(),
                        align_y: Vertical::Top,
                        ..CanvasText::default()
                    });
                }
            }
        }

        // 7. Interactive Scale Reference Silhouettes (1.8m Human & 4.5m Vehicle)
        if self
            .state
            .visible_layers
            .contains(&BlueprintLayer::NavigationScale)
        {
            let anchors = &self.project.bible.scale_anchors;
            for marker in &bp.scale_markers {
                let mp = self.state.world_to_screen(marker.x_m, marker.y_m);
                let is_selected = self
                    .state
                    .selected_entity_id
                    .as_deref()
                    .is_some_and(|s| s == marker.id);
                match marker.kind {
                    ScaleMarkerKind::Human => {
                        let rad = (0.35 * z).max(6.0);
                        frame.fill(
                            &Path::circle(mp, rad),
                            Color::from_rgba(0.243, 0.804, 0.588, 0.35),
                        );
                        frame.stroke(
                            &Path::circle(mp, rad),
                            Stroke {
                                width: if is_selected { 2.4 } else { 1.6 },
                                style: canvas::Style::Solid(p.success),
                                ..Stroke::default()
                            },
                        );
                        frame.fill(&Path::circle(mp, (rad * 0.42).max(2.5)), p.success);
                        frame.fill_text(CanvasText {
                            content: format!("{:.2}m HUMAN", anchors.human_height_m),
                            position: Point::new(mp.x, mp.y - rad - 4.0),
                            color: p.success,
                            size: 9.5.into(),
                            font: mono_font,
                            align_x: Horizontal::Center.into(),
                            align_y: Vertical::Bottom,
                            ..CanvasText::default()
                        });
                    }
                    ScaleMarkerKind::Vehicle => {
                        let vw = (anchors.vehicle_length_m as f32) * z;
                        let vh = (anchors.vehicle_width_m as f32) * z;
                        let tl = Point::new(mp.x - vw * 0.5, mp.y - vh * 0.5);
                        frame.fill_rectangle(
                            tl,
                            Size::new(vw, vh),
                            Color::from_rgba(0.965, 0.714, 0.282, 0.25),
                        );
                        frame.stroke(
                            &Path::rectangle(tl, Size::new(vw, vh)),
                            Stroke {
                                width: if is_selected { 2.4 } else { 1.5 },
                                style: canvas::Style::Solid(p.warning),
                                ..Stroke::default()
                            },
                        );
                        frame.fill_text(CanvasText {
                            content: format!(
                                "{:.1}×{:.1}m VEH",
                                anchors.vehicle_length_m, anchors.vehicle_width_m
                            ),
                            position: mp,
                            color: p.warning,
                            size: 9.0.into(),
                            font: mono_font,
                            align_x: Horizontal::Center.into(),
                            align_y: Vertical::Center,
                            ..CanvasText::default()
                        });
                    }
                }
            }
        }

        // 8. Asset ID Map Layer
        if self
            .state
            .visible_layers
            .contains(&BlueprintLayer::AssetIdMap)
        {
            for fp in &bp.footprints {
                let tl = self.state.world_to_screen(fp.rect.x_m, fp.rect.y_m);
                let tag_col = if violated_ids.contains(fp.id.as_str()) {
                    p.danger
                } else {
                    p.accent_text
                };
                frame.fill_text(CanvasText {
                    content: format!("{} [{}]", fp.id, fp.asset_id),
                    position: Point::new(tl.x + 2.0, tl.y - 3.0),
                    color: tag_col,
                    size: 9.5.into(),
                    font: mono_font,
                    align_x: Horizontal::Left.into(),
                    align_y: Vertical::Bottom,
                    ..CanvasText::default()
                });
            }
            for pl in &bp.placements {
                let pp = self.state.world_to_screen(pl.x_m, pl.y_m);
                frame.fill(&Path::circle(pp, 4.0), p.accent);
                frame.fill_text(CanvasText {
                    content: format!("{} ({})", pl.instance_id, pl.asset_id),
                    position: Point::new(pp.x + 6.0, pp.y - 4.0),
                    color: p.accent_text,
                    size: 9.0.into(),
                    font: mono_font,
                    align_x: Horizontal::Left.into(),
                    align_y: Vertical::Bottom,
                    ..CanvasText::default()
                });
            }
        }

        // 9. Active Distance Measure Ruler Line
        if let Some(((ax, ay), (bx, by))) = self.state.measure_line {
            let sa = self.state.world_to_screen(ax, ay);
            let sb = self.state.world_to_screen(bx, by);
            frame.stroke(
                &Path::line(sa, sb),
                Stroke {
                    width: 2.2,
                    style: canvas::Style::Solid(p.accent_text),
                    line_dash: canvas::LineDash {
                        segments: &[6.0, 3.0],
                        offset: 0,
                    },
                    ..Stroke::default()
                },
            );
            frame.fill(&Path::circle(sa, 4.0), p.accent_text);
            frame.fill(&Path::circle(sb, 4.0), p.accent_text);
            let dist = (bx - ax).hypot(by - ay);
            let mid = Point::new((sa.x + sb.x) * 0.5, (sa.y + sb.y) * 0.5 - 12.0);
            frame.fill_text(CanvasText {
                content: format!(
                    "{dist:.2}m (ΔX {:.2} · ΔY {:.2})",
                    (bx - ax).abs(),
                    (by - ay).abs()
                ),
                position: mid,
                color: p.text_primary,
                size: 11.0.into(),
                font: mono_font,
                align_x: Horizontal::Center.into(),
                align_y: Vertical::Bottom,
                ..CanvasText::default()
            });
        }

        // 10. Top & Left Metric Rulers + Scale Bar + Live Cursor Readout
        draw_rulers_and_scale_bar(&mut frame, bounds.size(), self.state, p, mono_font);

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        interaction: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if interaction.dragging_marker_id.is_some()
            || interaction.dragging_entity_id.is_some()
            || interaction.dragging_pan
        {
            return mouse::Interaction::Grabbing;
        }
        if let Some(local) = cursor.position_in(bounds)
            && local.x > RULER_LEFT
            && local.y > RULER_TOP
        {
            return match self.state.tool_mode {
                CanvasToolMode::SelectPan => mouse::Interaction::Pointer,
                CanvasToolMode::MoveEntity => mouse::Interaction::Grab,
                CanvasToolMode::MeasureRuler => mouse::Interaction::Crosshair,
            };
        }
        mouse::Interaction::default()
    }
}

pub fn view_blueprint_canvas<'a, M: 'a>(
    project: &'a LutetianProject,
    state: &'a BlueprintCanvasState,
    mode: UiThemeMode,
    map_msg: impl Fn(CanvasMsg) -> M + 'a,
) -> Element<'a, M> {
    let el: Element<'a, CanvasMsg> = canvas::Canvas::new(BlueprintCadProgram {
        project,
        state,
        mode,
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .into();
    el.map(map_msg)
}

fn snap_quarter(x: f64, y: f64) -> (f64, f64) {
    ((x * 4.0).round() / 4.0, (y * 4.0).round() / 4.0)
}

fn hit_test_movable(project: &LutetianProject, wx: f64, wy: f64) -> Option<String> {
    let bp = &project.blueprint;
    for door in &bp.doors {
        if (wx - door.position_m.x_m).hypot(wy - door.position_m.y_m) <= door.width_m.max(1.0) {
            return Some(door.id.clone());
        }
    }
    for fp in bp.footprints.iter().rev() {
        let (x0, y0, x1, y1) = fp.rect.world_aabb();
        if wx >= x0 && wx <= x1 && wy >= y0 && wy <= y1 {
            return Some(fp.id.clone());
        }
    }
    None
}

fn hit_test_entity(project: &LutetianProject, wx: f64, wy: f64) -> Option<String> {
    let bp = &project.blueprint;
    for marker in &bp.scale_markers {
        if (wx - marker.x_m).hypot(wy - marker.y_m) <= 1.2 {
            return Some(marker.id.clone());
        }
    }
    for door in &bp.doors {
        if (wx - door.position_m.x_m).hypot(wy - door.position_m.y_m) <= door.width_m.max(1.0) {
            return Some(door.id.clone());
        }
    }
    for fp in bp.footprints.iter().rev() {
        let (x0, y0, x1, y1) = fp.rect.world_aabb();
        if wx >= x0 && wx <= x1 && wy >= y0 && wy <= y1 {
            return Some(fp.id.clone());
        }
    }
    for zone in bp.zones.iter().rev() {
        let (x0, y0, x1, y1) = zone.rect.world_aabb();
        if wx >= x0 && wx <= x1 && wy >= y0 && wy <= y1 {
            return Some(zone.id.clone());
        }
    }
    None
}

fn rotated_rect_path(state: &BlueprintCanvasState, rect: &lutetian_core::MetricRect2D) -> Path {
    let c = rect.center();
    let hw = rect.width_m * 0.5;
    let hd = rect.depth_m * 0.5;
    let rad = rect.rotation_deg.to_radians();
    let cos = rad.cos();
    let sin = rad.sin();
    let corners = [(-hw, -hd), (hw, -hd), (hw, hd), (-hw, hd)];
    Path::new(|builder| {
        for (idx, (lx, ly)) in corners.iter().enumerate() {
            let wx = c.x_m + lx * cos - ly * sin;
            let wy = c.y_m + lx * sin + ly * cos;
            let sp = state.world_to_screen(wx, wy);
            if idx == 0 {
                builder.move_to(sp);
            } else {
                builder.line_to(sp);
            }
        }
        builder.close();
    })
}

fn zone_color(kind: ZoneSurfaceKind, mode: UiThemeMode) -> Color {
    let a = if mode == UiThemeMode::Dark {
        0.16
    } else {
        0.19
    };
    match kind {
        ZoneSurfaceKind::Grass => Color::from_rgba(0.24, 0.68, 0.42, a),
        ZoneSurfaceKind::Road => Color::from_rgba(0.48, 0.46, 0.54, a + 0.06),
        ZoneSurfaceKind::Dirt => Color::from_rgba(0.66, 0.48, 0.30, a),
        ZoneSurfaceKind::Water => Color::from_rgba(0.22, 0.58, 0.88, a + 0.05),
        ZoneSurfaceKind::Residential => Color::from_rgba(0.68, 0.52, 0.78, a),
        ZoneSurfaceKind::Commercial => Color::from_rgba(0.86, 0.64, 0.28, a),
        ZoneSurfaceKind::Plaza => Color::from_rgba(0.56, 0.54, 0.66, a),
    }
}

fn footprint_colors(kind: FootprintKind, mode: UiThemeMode, p: ThemePalette) -> (Color, Color) {
    let alpha = if mode == UiThemeMode::Dark {
        0.38
    } else {
        0.28
    };
    match kind {
        FootprintKind::Building => (
            Color::from_rgba(p.accent.r, p.accent.g, p.accent.b, alpha + 0.12),
            p.accent_text,
        ),
        FootprintKind::Wall => (Color::from_rgba(0.60, 0.62, 0.68, alpha), p.text_secondary),
        FootprintKind::Foliage => (Color::from_rgba(0.243, 0.804, 0.588, alpha), p.success),
        FootprintKind::Prop | FootprintKind::Vehicle => {
            (Color::from_rgba(0.965, 0.714, 0.282, alpha), p.warning)
        }
        FootprintKind::Bridge | FootprintKind::Other => {
            (Color::from_rgba(0.45, 0.65, 0.90, alpha), p.text_primary)
        }
    }
}

fn draw_metric_grid(
    frame: &mut Frame,
    size: Size,
    state: &BlueprintCanvasState,
    world_w_m: f64,
    world_d_m: f64,
    grid_unit_m: f64,
    p: ThemePalette,
) {
    let step = grid_unit_m.max(0.5);
    let z = state.zoom_px_per_m;
    let minor_col = Color::from_rgba(
        p.border_subtle.r,
        p.border_subtle.g,
        p.border_subtle.b,
        0.05,
    );
    let major_col = Color::from_rgba(
        p.border_subtle.r,
        p.border_subtle.g,
        p.border_subtle.b,
        0.14,
    );

    let (min_wx, min_wy) = state.screen_to_world(Point::new(RULER_LEFT, RULER_TOP));
    let (max_wx, max_wy) = state.screen_to_world(Point::new(size.width, size.height));

    let start_x = ((min_wx / step).floor() as i32).max(-100);
    let end_x = ((max_wx / step).ceil() as i32).min(400);
    for ix in start_x..=end_x {
        let wx = (ix as f64) * step;
        let sx = RULER_LEFT + state.pan_x_px + (wx as f32) * z;
        if sx < RULER_LEFT || sx > size.width {
            continue;
        }
        let is_major = ix % 5 == 0;
        if z < 8.0 && !is_major {
            continue;
        }
        frame.stroke(
            &Path::line(Point::new(sx, RULER_TOP), Point::new(sx, size.height)),
            Stroke {
                width: 1.0,
                style: canvas::Style::Solid(if is_major { major_col } else { minor_col }),
                ..Stroke::default()
            },
        );
    }

    let start_y = ((min_wy / step).floor() as i32).max(-100);
    let end_y = ((max_wy / step).ceil() as i32).min(400);
    for iy in start_y..=end_y {
        let wy = (iy as f64) * step;
        let sy = RULER_TOP + state.pan_y_px + (wy as f32) * z;
        if sy < RULER_TOP || sy > size.height {
            continue;
        }
        let is_major = iy % 5 == 0;
        if z < 8.0 && !is_major {
            continue;
        }
        frame.stroke(
            &Path::line(Point::new(RULER_LEFT, sy), Point::new(size.width, sy)),
            Stroke {
                width: 1.0,
                style: canvas::Style::Solid(if is_major { major_col } else { minor_col }),
                ..Stroke::default()
            },
        );
    }

    let w_tl = state.world_to_screen(0.0, 0.0);
    let w_sz = Size::new((world_w_m as f32) * z, (world_d_m as f32) * z);
    frame.stroke(
        &Path::rectangle(w_tl, w_sz),
        Stroke {
            width: 1.6,
            style: canvas::Style::Solid(p.border_focus),
            ..Stroke::default()
        },
    );
}

fn draw_rulers_and_scale_bar(
    frame: &mut Frame,
    size: Size,
    state: &BlueprintCanvasState,
    p: ThemePalette,
    mono_font: Font,
) {
    frame.fill_rectangle(
        Point::ORIGIN,
        Size::new(size.width, RULER_TOP),
        p.bg_sidebar,
    );
    frame.fill_rectangle(
        Point::ORIGIN,
        Size::new(RULER_LEFT, size.height),
        p.bg_sidebar,
    );
    frame.stroke(
        &Path::line(
            Point::new(0.0, RULER_TOP),
            Point::new(size.width, RULER_TOP),
        ),
        Stroke {
            width: 1.0,
            style: canvas::Style::Solid(p.border_subtle),
            ..Stroke::default()
        },
    );
    frame.stroke(
        &Path::line(
            Point::new(RULER_LEFT, 0.0),
            Point::new(RULER_LEFT, size.height),
        ),
        Stroke {
            width: 1.0,
            style: canvas::Style::Solid(p.border_subtle),
            ..Stroke::default()
        },
    );
    frame.fill_text(CanvasText {
        content: "m".to_string(),
        position: Point::new(RULER_LEFT * 0.5, RULER_TOP * 0.5),
        color: p.accent_text,
        size: 10.0.into(),
        font: mono_font,
        align_x: Horizontal::Center.into(),
        align_y: Vertical::Center,
        ..CanvasText::default()
    });

    let z = state.zoom_px_per_m;
    let tick_step_m: i32 = if z >= 22.0 {
        2
    } else if z >= 9.0 {
        5
    } else {
        10
    };

    let (min_wx, min_wy) = state.screen_to_world(Point::new(RULER_LEFT, RULER_TOP));
    let (max_wx, max_wy) = state.screen_to_world(Point::new(size.width, size.height));

    let sx0 = (min_wx / (tick_step_m as f64)).floor() as i32 * tick_step_m;
    let sx1 = (max_wx / (tick_step_m as f64)).ceil() as i32 * tick_step_m;
    let mut x = sx0;
    while x <= sx1 {
        let px = RULER_LEFT + state.pan_x_px + (x as f32) * z;
        if px >= RULER_LEFT + 6.0 && px <= size.width - 8.0 {
            frame.stroke(
                &Path::line(Point::new(px, RULER_TOP - 5.0), Point::new(px, RULER_TOP)),
                Stroke {
                    width: 1.0,
                    style: canvas::Style::Solid(p.text_muted),
                    ..Stroke::default()
                },
            );
            frame.fill_text(CanvasText {
                content: format!("{x}"),
                position: Point::new(px, RULER_TOP - 15.0),
                color: p.text_secondary,
                size: 9.5.into(),
                font: mono_font,
                align_x: Horizontal::Center.into(),
                align_y: Vertical::Center,
                ..CanvasText::default()
            });
        }
        x += tick_step_m;
    }

    let sy0 = (min_wy / (tick_step_m as f64)).floor() as i32 * tick_step_m;
    let sy1 = (max_wy / (tick_step_m as f64)).ceil() as i32 * tick_step_m;
    let mut y = sy0;
    while y <= sy1 {
        let py = RULER_TOP + state.pan_y_px + (y as f32) * z;
        if py >= RULER_TOP + 6.0 && py <= size.height - 8.0 {
            frame.stroke(
                &Path::line(Point::new(RULER_LEFT - 5.0, py), Point::new(RULER_LEFT, py)),
                Stroke {
                    width: 1.0,
                    style: canvas::Style::Solid(p.text_muted),
                    ..Stroke::default()
                },
            );
            frame.fill_text(CanvasText {
                content: format!("{y}"),
                position: Point::new(RULER_LEFT - 8.0, py),
                color: p.text_secondary,
                size: 9.5.into(),
                font: mono_font,
                align_x: Horizontal::Right.into(),
                align_y: Vertical::Center,
                ..CanvasText::default()
            });
        }
        y += tick_step_m;
    }

    // Bottom-left live cursor coordinates pill
    if let Some((cx, cy)) = state.cursor_world_m {
        let coord_txt = format!("X {cx:.2}m · Y {cy:.2}m");
        frame.fill_rectangle(
            Point::new(RULER_LEFT + 10.0, size.height - 32.0),
            Size::new(152.0, 22.0),
            Color::from_rgba(p.bg_sidebar.r, p.bg_sidebar.g, p.bg_sidebar.b, 0.88),
        );
        frame.fill_text(CanvasText {
            content: coord_txt,
            position: Point::new(RULER_LEFT + 18.0, size.height - 21.0),
            color: p.accent_text,
            size: 10.0.into(),
            font: mono_font,
            align_x: Horizontal::Left.into(),
            align_y: Vertical::Center,
            ..CanvasText::default()
        });
    }

    // Bottom-right scale bar
    let bar_meters = if z >= 14.0 { 5.0 } else { 10.0 };
    let bar_px = bar_meters * z;
    let bx = (size.width - bar_px - 22.0).max(RULER_LEFT + 174.0);
    let by = size.height - 20.0;
    frame.fill_rectangle(
        Point::new(bx - 10.0, by - 22.0),
        Size::new(bar_px + 20.0, 32.0),
        Color::from_rgba(p.bg_sidebar.r, p.bg_sidebar.g, p.bg_sidebar.b, 0.88),
    );
    frame.stroke(
        &Path::new(|b| {
            b.move_to(Point::new(bx, by - 4.0));
            b.line_to(Point::new(bx, by));
            b.line_to(Point::new(bx + bar_px, by));
            b.line_to(Point::new(bx + bar_px, by - 4.0));
        }),
        Stroke {
            width: 1.6,
            style: canvas::Style::Solid(p.accent_text),
            ..Stroke::default()
        },
    );
    frame.fill_text(CanvasText {
        content: format!("{:.0}m · {:.1} px/m", bar_meters, z),
        position: Point::new(bx + bar_px * 0.5, by - 12.0),
        color: p.text_primary,
        size: 9.5.into(),
        font: mono_font,
        align_x: Horizontal::Center.into(),
        align_y: Vertical::Center,
        ..CanvasText::default()
    });
}
