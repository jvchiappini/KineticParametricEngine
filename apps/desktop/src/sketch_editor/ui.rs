use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::app::AppState;
use kpe_geometry::sketch::constraints::Constraint;
use kpe_geometry::sketch::entities::EntityId;
use kpe_schema::geometry::{GeometryNode, GeometryNodeType, SketchDef};
use super::state::{SketchEditorState, SketchTool, EntityKind, PendingExtrude, entity_label};
use super::solver;

pub fn sketch_ui(
    mut contexts: EguiContexts,
    mut editor: ResMut<SketchEditorState>,
) {
    if !editor.active { return; }

    egui::TopBottomPanel::top("sketch_toolbar")
        .min_height(36.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                ui.heading("Sketch");
                ui.separator();

                let tools = [SketchTool::Select, SketchTool::Line, SketchTool::Circle, SketchTool::Arc, SketchTool::Measure];
                let tool_names = ["Select", "Line", "Circle", "Arc", "Measure"];
                for (i, tool) in tools.iter().enumerate() {
                    if ui.selectable_label(editor.tool == *tool, tool_names[i]).clicked() {
                        editor.tool = *tool;
                        editor.line_start = None;
                        editor.circle_center = None;
                        editor.arc_center = None;
                        editor.measure_click_a = None;
                        editor.measure_click_b = None;
                        editor.measure_result = None;
                    }
                }

                ui.separator();
                let ids = editor.selected_ids();
                let kinds: Vec<EntityKind> = ids.iter().filter_map(|&id| editor.find_in_selection(id)).collect();
                ui.add_enabled(
                    kinds.len() >= 2 && kinds.iter().filter(|k| matches!(k, EntityKind::Point(_))).count() >= 2,
                    egui::Button::new("Coincident"),
                ).on_hover_text("Make 2 selected points coincident").clicked().then(|| {
                    let pts: Vec<EntityId> = kinds.iter().filter_map(|k| if let EntityKind::Point(id) = k { Some(*id) } else { None }).collect();
                    if pts.len() >= 2 {
                        editor.inject_constraint(Constraint::Coincident { point_a: pts[0], point_b: pts[1] });
                    }
                });
                ui.add_enabled(
                    kinds.iter().any(|k| matches!(k, EntityKind::Point(_))),
                    egui::Button::new("Fix"),
                ).on_hover_text("Fix selected point in place").clicked().then(|| {
                    if let Some(EntityKind::Point(pid)) = kinds.first() {
                        editor.inject_constraint(Constraint::Fix { point: *pid, x: 0.0, y: 0.0 });
                    }
                });
                ui.add_enabled(
                    kinds.iter().any(|k| matches!(k, EntityKind::Line(_)))
                        && kinds.iter().any(|k| matches!(k, EntityKind::Circle(_)) || matches!(k, EntityKind::Arc(_))),
                    egui::Button::new("Tangent"),
                ).on_hover_text("Make line tangent to circle or arc").clicked().then(|| {
                    let line = kinds.iter().find_map(|k| if let EntityKind::Line(id) = k { Some(*id) } else { None });
                    let arc_or_circle = kinds.iter().find_map(|k| match *k { EntityKind::Circle(id) | EntityKind::Arc(id) => Some(id), _ => None });
                    if let (Some(l), Some(a)) = (line, arc_or_circle) {
                        editor.inject_constraint(Constraint::Tangent { line: l, arc: a });
                    }
                });
                ui.add_enabled(
                    kinds.iter().filter(|k| matches!(k, EntityKind::Circle(_))).count() >= 2,
                    egui::Button::new("Concentric"),
                ).on_hover_text("Make 2 circles share a center").clicked().then(|| {
                    let cids: Vec<EntityId> = kinds.iter().filter_map(|k| if let EntityKind::Circle(id) = k { Some(*id) } else { None }).collect();
                    if cids.len() >= 2 {
                        let c1 = editor.document.circles.iter().find(|c| c.id == cids[0]).map(|c| c.center);
                        let c2 = editor.document.circles.iter().find(|c| c.id == cids[1]).map(|c| c.center);
                        if let (Some(pa), Some(pb)) = (c1, c2) {
                            editor.inject_constraint(Constraint::Coincident { point_a: pa, point_b: pb });
                        }
                    }
                });
                ui.separator();
                ui.checkbox(&mut editor.show_constraints, "Cst");
                ui.checkbox(&mut editor.grid_snap, "Snap")
                    .on_hover_text("Snap to grid (default 0.5)");

                ui.separator();
                let can_undo = editor.can_undo();
                if ui.add_enabled(can_undo, egui::Button::new("Undo")).clicked() {
                    editor.undo();
                }
                let can_redo = editor.can_redo();
                if ui.add_enabled(can_redo, egui::Button::new("Redo")).clicked() {
                    editor.redo();
                }

                ui.separator();
                ui.label("Extr:");
                ui.add(egui::DragValue::new(&mut editor.extrude_distance).speed(0.1).suffix("m"));
                let mut taper = editor.extrude_taper_angle != 0.0;
                if ui.checkbox(&mut taper, "Taper").changed() {
                    if !taper { editor.extrude_taper_angle = 0.0; }
                }
                if taper {
                    ui.add(egui::DragValue::new(&mut editor.extrude_taper_angle).speed(0.5).suffix("\u{b0}").range(-60.0..=60.0));
                }
                if ui.button("Extrude").clicked() {
                    editor.pending_extrude = Some(PendingExtrude {
                        distance: editor.extrude_distance,
                        taper_angle: editor.extrude_taper_angle,
                    });
                }

                if let Some(ref m) = editor.measure_result {
                    ui.label(m);
                    ui.separator();
                }
                if let Some(ref err) = editor.last_solve_error {
                    ui.colored_label(egui::Color32::RED, err);
                    ui.separator();
                }
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
        });

    let constraints: Vec<String> = editor.document.constraints.iter().map(describe_short).collect();
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
                    let editable = matches!(c, Constraint::Distance { .. } | Constraint::Angle { .. } | Constraint::Radius { .. });
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
                        ui.add(egui::DragValue::new(&mut val).speed(0.1).range(min..=max));
                        if ui.button("OK").clicked() {
                            editor.save_snapshot();
                            if let Some(c_mut) = editor.document.constraints.get_mut(idx) {
                                match c_mut {
                                    Constraint::Distance { ref mut distance, .. } => *distance = val,
                                    Constraint::Angle { ref mut angle, .. } => *angle = val,
                                    Constraint::Radius { ref mut radius, .. } => *radius = val,
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
}

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
