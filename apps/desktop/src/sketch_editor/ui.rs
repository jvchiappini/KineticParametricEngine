use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::app::AppState;
use kpe_geometry::sketch::constraints::Constraint;
use kpe_geometry::sketch::entities::EntityId;
use kpe_geometry::sketch::inference::SnapResult;
use kpe_schema::geometry::{GeometryNode, GeometryNodeType, SketchDef};
use super::state::{SketchEditorState, SketchTool, OsnapSettings,
                   EntityKind, PendingExtrude, ToolPhase, entity_label};
use super::solver;

// ── Helpers ──────────────────────────────────────────────────────────

/// Draw a mini toggle button in the snap strip.
fn snap_toggle(ui: &mut egui::Ui, label: &str, enabled: &mut bool) {
    let btn = egui::Button::new(label).min_size(egui::vec2(52.0, 22.0));
    let resp = if *enabled {
        ui.add(btn.fill(egui::Color32::from_rgb(60, 100, 180)))
    } else {
        ui.add(btn)
    };
    if resp.clicked() {
        *enabled = !*enabled;
    }
}

/// Draw the OSnap toggle strip — 6 small persistent buttons.
fn show_osnap_strip(ui: &mut egui::Ui, s: &mut OsnapSettings) {
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(2.0, 2.0);
        ui.label("OSnap:");
        snap_toggle(ui, "End", &mut s.endpoint);
        snap_toggle(ui, "Mid", &mut s.midpoint);
        snap_toggle(ui, "Ctr", &mut s.center);
        snap_toggle(ui, "Quad", &mut s.quadrant);
        snap_toggle(ui, "Int", &mut s.intersection);
        snap_toggle(ui, "On", &mut s.on_entity);
    });
}

/// Floating tooltip near the cursor showing command state and snap feedback.
fn show_cursor_tooltip(
    ctx: &egui::Context,
    tooltip: &str,
    snap: Option<&SnapResult>,
    cursor_2d: egui::Pos2,
) {
    if tooltip.is_empty() && snap.is_none() { return; }

    let mut lines: Vec<String> = Vec::new();
    if !tooltip.is_empty() {
        lines.push(tooltip.to_string());
    }
    if let Some(s) = snap {
        lines.push(format!("[{}] ({:.2}, {:.2})", s.kind, s.x, s.y));
    }

    let text = lines.join("\n");

    egui::Area::new(egui::Id::new("sketch_cursor_tooltip"))
        .fixed_pos(cursor_2d + egui::vec2(16.0, 18.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let frame = egui::Frame::none()
                .fill(egui::Color32::from_black_alpha(200))
                .stroke(egui::epaint::Stroke::new(1.0, egui::Color32::WHITE))
                .inner_margin(egui::Margin::symmetric(6.0, 3.0));
            frame.show(ui, |ui: &mut egui::Ui| {
                ui.label(egui::RichText::new(text).monospace().size(12.0));
            });
        });
}

// ── Main sketch_ui ───────────────────────────────────────────────────

pub fn sketch_ui(
    mut contexts: EguiContexts,
    mut editor: ResMut<SketchEditorState>,
) {
    if !editor.active { return; }

    // ── Top toolbar ──────────────────────────────────────────────────
    egui::TopBottomPanel::top("sketch_toolbar")
        .min_height(36.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                // ── ROW 1: Tool buttons ──
                ui.heading("Sketch");
                ui.separator();

                let tools = [
                    SketchTool::Select,
                    SketchTool::Line,
                    SketchTool::Circle,
                    SketchTool::Arc,
                    SketchTool::Measure,
                ];
                let tool_names = ["Select", "Line", "Circle", "Arc", "Measure"];
                for (i, tool) in tools.iter().enumerate() {
                    let is_active = editor.tool == *tool
                        || (editor.tool_phase != ToolPhase::Idle && *tool == SketchTool::Line);
                    if ui.selectable_label(is_active, tool_names[i]).clicked() {
                        editor.tool = *tool;
                        editor.line_start = None;
                        editor.circle_center = None;
                        editor.arc_center = None;
                        editor.measure_click_a = None;
                        editor.measure_click_b = None;
                        editor.measure_result = None;
                        editor.tool_phase = ToolPhase::Idle;
                        editor.snap_feedback = None;
                        editor.cursor_tooltip.clear();
                    }
                }

                // ── Constraint buttons ──
                ui.separator();
                let ids = editor.selected_ids();
                let kinds: Vec<EntityKind> = ids.iter()
                    .filter_map(|&id| editor.find_in_selection(id)).collect();
                ui.add_enabled(
                    kinds.len() >= 2 && kinds.iter()
                        .filter(|k| matches!(k, EntityKind::Point(_))).count() >= 2,
                    egui::Button::new("Coincident"),
                ).on_hover_text("Make 2 selected points coincident").clicked().then(|| {
                    let pts: Vec<EntityId> = kinds.iter()
                        .filter_map(|k| if let EntityKind::Point(id) = k { Some(*id) } else { None })
                        .collect();
                    if pts.len() >= 2 {
                        editor.inject_constraint(
                            Constraint::Coincident { point_a: pts[0], point_b: pts[1] });
                    }
                });
                ui.add_enabled(
                    kinds.iter().any(|k| matches!(k, EntityKind::Point(_))),
                    egui::Button::new("Fix"),
                ).on_hover_text("Fix selected point in place").clicked().then(|| {
                    if let Some(EntityKind::Point(pid)) = kinds.first() {
                        editor.inject_constraint(
                            Constraint::Fix { point: *pid, x: 0.0, y: 0.0 });
                    }
                });
                ui.add_enabled(
                    kinds.iter().any(|k| matches!(k, EntityKind::Line(_)))
                        && kinds.iter().any(|k| matches!(k, EntityKind::Circle(_))
                            || matches!(k, EntityKind::Arc(_))),
                    egui::Button::new("Tangent"),
                ).on_hover_text("Make line tangent to circle or arc").clicked().then(|| {
                    let line = kinds.iter().find_map(|k|
                        if let EntityKind::Line(id) = k { Some(*id) } else { None });
                    let arc_or_circle = kinds.iter().find_map(|k| match *k {
                        EntityKind::Circle(id) | EntityKind::Arc(id) => Some(id),
                        _ => None,
                    });
                    if let (Some(l), Some(a)) = (line, arc_or_circle) {
                        editor.inject_constraint(
                            Constraint::Tangent { line: l, arc: a });
                    }
                });
                ui.add_enabled(
                    kinds.iter().filter(|k| matches!(k, EntityKind::Circle(_))).count() >= 2,
                    egui::Button::new("Concentric"),
                ).on_hover_text("Make 2 circles share a center").clicked().then(|| {
                    let cids: Vec<EntityId> = kinds.iter()
                        .filter_map(|k| if let EntityKind::Circle(id) = k { Some(*id) } else { None })
                        .collect();
                    if cids.len() >= 2 {
                        let c1 = editor.document.circles.iter()
                            .find(|c| c.id == cids[0]).map(|c| c.center);
                        let c2 = editor.document.circles.iter()
                            .find(|c| c.id == cids[1]).map(|c| c.center);
                        if let (Some(pa), Some(pb)) = (c1, c2) {
                            editor.inject_constraint(
                                Constraint::Coincident { point_a: pa, point_b: pb });
                        }
                    }
                });

                // ── Extrude ──
                ui.separator();
                ui.label("Extr:");
                ui.add(egui::DragValue::new(&mut editor.extrude_distance)
                    .speed(0.1).suffix("m"));
                let mut taper = editor.extrude_taper_angle != 0.0;
                if ui.checkbox(&mut taper, "Taper").changed() {
                    if !taper { editor.extrude_taper_angle = 0.0; }
                }
                if taper {
                    ui.add(egui::DragValue::new(&mut editor.extrude_taper_angle)
                        .speed(0.5).suffix("\u{b0}").range(-60.0..=60.0));
                }
                if ui.button("Extrude").clicked() {
                    editor.pending_extrude = Some(PendingExtrude {
                        distance: editor.extrude_distance,
                        taper_angle: editor.extrude_taper_angle,
                    });
                }

                // ── Status / undo ──
                ui.separator();
                ui.add_enabled(editor.can_undo(), egui::Button::new("Undo"))
                    .clicked().then(|| editor.undo());
                ui.add_enabled(editor.can_redo(), egui::Button::new("Redo"))
                    .clicked().then(|| editor.redo());

                if let Some(ref m) = editor.measure_result {
                    ui.separator();
                    ui.label(m);
                }
                if let Some(ref err) = editor.last_solve_error {
                    ui.separator();
                    ui.colored_label(egui::Color32::RED, err);
                }
                if let Some(ref h) = editor.hovered_entity {
                    ui.separator();
                    ui.label(format!("Hover: {}", h));
                }
                ui.separator();
                ui.label(entity_label(&editor));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Finish").clicked() {
                        editor.pending_finish = true;
                    }
                    if ui.button("Cancel").clicked() {
                        editor.pending_cancel = true;
                    }
                });
            });

            // ── ROW 2: Snap strip ──
            ui.separator();
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);

                // Grid snap
                ui.checkbox(&mut editor.grid_snap, "Grid")
                    .on_hover_text("Snap to grid (default 0.5)");
                if editor.grid_snap {
                    ui.add(egui::DragValue::new(&mut editor.snap_size)
                        .speed(0.05).range(0.05..=10.0).suffix("m"));
                }

                ui.separator();

                // Ortho (F8)
                let ortho_resp = if editor.ortho_enabled {
                    ui.selectable_label(true, "Ortho [F8]")
                        .highlight()
                } else {
                    ui.selectable_label(false, "Ortho [F8]")
                };
                if ortho_resp.clicked() {
                    editor.ortho_enabled = !editor.ortho_enabled;
                }

                // Polar toggle
                let polar_resp = if editor.polar_enabled {
                    ui.selectable_label(true, "Polar")
                        .highlight()
                } else {
                    ui.selectable_label(false, "Polar")
                };
                if polar_resp.clicked() {
                    editor.polar_enabled = !editor.polar_enabled;
                }

                ui.separator();

                // OSnap toggle strip
                show_osnap_strip(ui, &mut editor.osnap_settings);

                ui.separator();

                // Show constraints toggle
                ui.checkbox(&mut editor.show_constraints, "Cst");
            });
        });

    // ── Constraints side panel ──────────────────────────────────────
    let constraints: Vec<String> = editor.document.constraints.iter()
        .map(describe_short).collect();
    let edit_idx = editor.editing_constraint_idx;
    egui::SidePanel::right("sketch_constraints")
        .resizable(true)
        .default_width(180.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.add_space(8.0);
            ui.heading("Constraints");
            ui.separator();
            let mut delete_cst: Option<usize> = None;
            let mut clicked_cst: Option<usize> = None;
            for (i, label) in constraints.iter().enumerate() {
                let is_selected = edit_idx == Some(i);
                let r = ui.selectable_label(is_selected, label);
                if r.clicked() {
                    clicked_cst = Some(i);
                }
                r.context_menu(|ui| {
                    if ui.button("Delete").clicked() {
                        delete_cst = Some(i);
                        ui.close_menu();
                    }
                });
            }
            if let Some(idx) = clicked_cst {
                if let Some(c) = editor.document.constraints.get(idx) {
                    let editable = matches!(c, Constraint::Distance { .. }
                        | Constraint::Angle { .. } | Constraint::Radius { .. });
                    if editable {
                        let val = match *c {
                            Constraint::Distance { distance, .. } => distance,
                            Constraint::Angle { angle, .. } => angle,
                            Constraint::Radius { radius, .. } => radius,
                            _ => 0.0,
                        };
                        editor.editing_constraint_idx = Some(idx);
                        editor.editing_new_value = val;
                    }
                }
            }
            if let Some(idx) = delete_cst {
                editor.save_snapshot();
                editor.document.constraints.remove(idx);
                solver::solve_sync(&mut editor);
                if editor.editing_constraint_idx == Some(idx) {
                    editor.editing_constraint_idx = None;
                }
            }
        });

    // ── Edit constraint dialog ──────────────────────────────────────
    if let Some(idx) = editor.editing_constraint_idx {
        if let Some(c) = editor.document.constraints.get(idx) {
            let c = c.clone();
            let label = format!("Edit {}", describe_short(&c));
            egui::Window::new(label)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(contexts.ctx_mut(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Value:");
                        let (min, max) = match &c {
                            Constraint::Distance { .. } => (0.001, 10000.0),
                            Constraint::Angle { .. } => (-360.0, 360.0),
                            Constraint::Radius { .. } => (0.001, 10000.0),
                            _ => (f64::NEG_INFINITY, f64::INFINITY),
                        };
                        let mut val = match &c {
                            Constraint::Distance { distance, .. } => *distance,
                            Constraint::Angle { angle, .. } => *angle,
                            Constraint::Radius { radius, .. } => *radius,
                            _ => 0.0,
                        };
                        ui.add(egui::DragValue::new(&mut val)
                            .speed(0.1).range(min..=max));
                        if ui.button("OK").clicked() {
                            editor.save_snapshot();
                            if let Some(c_mut) = editor.document.constraints.get_mut(idx) {
                                match c_mut {
                                    Constraint::Distance { ref mut distance, .. }
                                        => *distance = val,
                                    Constraint::Angle { ref mut angle, .. }
                                        => *angle = val,
                                    Constraint::Radius { ref mut radius, .. }
                                        => *radius = val,
                                    _ => {}
                                }
                            }
                            solver::solve_sync(&mut editor);
                            editor.editing_constraint_idx = None;
                        }
                        if ui.button("Cancel").clicked() {
                            editor.editing_constraint_idx = None;
                        }
                    });
                });
        } else {
            editor.editing_constraint_idx = None;
        }
    }

    // ── Tool Property Inspector ──────────────────────────────────────
    if editor.active && editor.tool != SketchTool::Select {
        let inspector_label = match editor.tool {
            SketchTool::Line => "Line Options",
            SketchTool::Circle => "Circle Options",
            SketchTool::Arc => "Arc Options",
            _ => "Options",
        };
        egui::Window::new(inspector_label)
            .anchor(egui::Align2::RIGHT_TOP, [-10.0, 50.0])
            .resizable(false)
            .collapsible(true)
            .default_width(180.0)
            .show(contexts.ctx_mut(), |ui| {
                match editor.tool {
                    SketchTool::Line => {
                        let mut dist = editor.dynamic_input.distance.unwrap_or(0.0);
                        let mut ang = editor.dynamic_input.angle_deg.unwrap_or(0.0);
                        ui.label("Distance:");
                        if ui.add(egui::DragValue::new(&mut dist)
                            .speed(0.1).suffix("m").range(0.001..=10000.0)).changed()
                        {
                            editor.dynamic_input.set_distance(dist);
                        }
                        ui.label("Angle:");
                        if ui.add(egui::DragValue::new(&mut ang)
                            .speed(1.0).suffix("\u{b0}").range(-360.0..=360.0)).changed()
                        {
                            editor.dynamic_input.set_angle_deg(ang);
                        }
                        if ui.button("Clear Overrides").clicked() {
                            editor.dynamic_input.clear();
                        }
                    }
                    SketchTool::Circle => {
                        ui.checkbox(&mut editor.circle_diameter_mode, "Diameter mode");
                        if let Some((cx, cy)) = editor.circle_center {
                            let (cur_x, cur_y) = editor.snap_feedback
                                .as_ref()
                                .map(|s| (s.x, s.y))
                                .unwrap_or((0.0, 0.0));
                            let r = ((cur_x - cx).powi(2) + (cur_y - cy).powi(2)).sqrt();
                            if editor.circle_diameter_mode {
                                ui.label(format!("Diameter: {:.3}m", r * 2.0));
                            } else {
                                ui.label(format!("Radius: {:.3}m", r));
                            }
                        }
                    }
                    SketchTool::Arc => {
                        if let Some((cx, cy)) = editor.arc_center {
                            let (cur_x, cur_y) = editor.snap_feedback
                                .as_ref()
                                .map(|s| (s.x, s.y))
                                .unwrap_or((0.0, 0.0));
                            let r = ((cur_x - cx).powi(2) + (cur_y - cy).powi(2)).sqrt();
                            let angle = (cur_y - cy).atan2(cur_x - cx).to_degrees();
                            ui.label(format!("Radius: {:.3}m", r));
                            ui.label(format!("Angle: {:.1}\u{b0}", angle));
                        }
                    }
                    _ => {}
                }
            });
    }

    // ── Cursor tooltip ───────────────────────────────────────────────
    let ctx = contexts.ctx_mut();
    if let Some(cursor) = ctx.pointer_latest_pos() {
        show_cursor_tooltip(
            ctx,
            &editor.cursor_tooltip,
            editor.snap_feedback.as_ref(),
            cursor,
        );
    }
}

// ── Helpers ──────────────────────────────────────────────────────────

fn describe_short(c: &Constraint) -> String {
    match *c {
        Constraint::Distance { distance, .. } => format!("Distance ({:.2})", distance),
        Constraint::Angle { angle, .. } => format!("Angle ({:.2}\u{b0})", angle),
        Constraint::Radius { radius, .. } => format!("Radius ({:.2})", radius),
        Constraint::Horizontal { .. } => "Horizontal".into(),
        Constraint::Vertical { .. } => "Vertical".into(),
        Constraint::EqualLength { .. } => "Equal Length".into(),
        Constraint::Parallel { .. } => "Parallel".into(),
        Constraint::Perpendicular { .. } => "Perpendicular".into(),
        Constraint::Collinear { .. } => "Collinear".into(),
        Constraint::Coincident { .. } => "Coincident".into(),
        Constraint::Fix { .. } => "Fix".into(),
        _ => "Constraint".into(),
    }
}

pub fn check_enter_sketch_mode(
    mut state: ResMut<AppState>,
    mut editor: ResMut<SketchEditorState>,
) {
    let Some(node_id) = state.pending_sketch_edit.take() else { return };
    let sketch_def = find_sketch_def(&state.document.recipe.scene, &node_id);
    if let Some(def) = sketch_def {
        editor.enter(&node_id, &def);
    }
}

fn find_sketch_def(node: &GeometryNode, target: &str) -> Option<SketchDef> {
    if node.id == target {
        if let GeometryNodeType::Sketch(ref def) = node.node_type {
            return Some(def.clone());
        }
        return None;
    }
    for child in &node.children {
        if let result @ Some(_) = find_sketch_def(child, target) {
            return result;
        }
    }
    None
}
