use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::app::AppState;
use crate::camera::OrbitCamera;
use kpe_geometry::sketch::document::SketchDocument;
use kpe_geometry::sketch::entities::{EntityId, closest_point_on_line};
use kpe_geometry::sketch::constraints::Constraint;
use kpe_schema::geometry::{GeometryNode, SketchDef, SketchPlane, SketchPrimitive, GeometryNodeType, ExtrudeDef};

pub(crate) fn to_3d(x: f64, y: f64, plane: &SketchPlane) -> Vec3 {
    match plane {
        SketchPlane::XY => Vec3::new(x as f32, y as f32, 0.0),
        SketchPlane::XZ => Vec3::new(x as f32, 0.0, y as f32),
        SketchPlane::YZ => Vec3::new(0.0, x as f32, y as f32),
    }
}

fn to_2d(pos: Vec3, plane: &SketchPlane) -> (f64, f64) {
    match plane {
        SketchPlane::XY => (pos.x as f64, pos.y as f64),
        SketchPlane::XZ => (pos.x as f64, pos.z as f64),
        SketchPlane::YZ => (pos.y as f64, pos.z as f64),
    }
}

pub(crate) fn sketch_plane_normal(plane: &SketchPlane) -> Dir3 {
    match plane {
        SketchPlane::XY => Dir3::Z,
        SketchPlane::XZ => Dir3::Y,
        SketchPlane::YZ => Dir3::X,
    }
}

pub(crate) fn circle_basis(normal: Dir3) -> (Vec3, Vec3) {
    let n = normal.as_vec3();
    let ref_vec = if n.abs().x > 0.9 { Vec3::Y } else { Vec3::X };
    let right = n.cross(ref_vec).normalize();
    let forward = n.cross(right).normalize();
    (right, forward)
}

fn pick_point(pos: &[(EntityId, f64, f64)], x: f64, y: f64, threshold: f64) -> Option<EntityId> {
    let pa = glam::DVec2::new(x, y);
    pos.iter()
        .map(|(id, px, py)| (*id, glam::DVec2::new(*px, *py)))
        .min_by(|(_, a), (_, b)| a.distance(pa).partial_cmp(&b.distance(pa)).unwrap())
        .filter(|(_, p)| p.distance(pa) < threshold)
        .map(|(id, _)| id)
}

fn entity_label(editor: &SketchEditorState) -> String {
    format!("{}L {}C {}A {}P {}Cst",
        editor.document.lines.len(),
        editor.document.circles.len(),
        editor.document.arcs.len(),
        editor.document.points.len(),
        editor.document.constraints.len(),
    )
}

#[derive(Resource)]
pub struct SketchEditorState {
    pub active: bool,
    pub node_id: String,
    pub document: SketchDocument,
    pub tool: SketchTool,
    pub selected_entity: Option<EntityId>,
    pub selected_entities: Vec<EntityId>,
    pub grid_size: f64,
    pub plane: SketchPlane,
    pub line_start: Option<(f64, f64)>,
    pub drag_from: Option<EntityId>,
    pub drag_offset: glam::DVec2,
    pub drag_start_saved: bool,
    pub show_constraints: bool,
    pub extrude_distance: f64,
    pub extrude_taper_angle: f64,
    pub circle_center: Option<(f64, f64)>,
    pub arc_center: Option<(f64, f64)>,
    pub measure_click_a: Option<(f64, f64)>,
    pub measure_click_b: Option<(f64, f64)>,
    pub measure_result: Option<String>,
    pub editing_constraint_idx: Option<usize>,
    pub editing_new_value: f64,
    pub grid_snap: bool,
    pub snap_size: f64,
    undo_stack: Vec<SketchDocument>,
    redo_stack: Vec<SketchDocument>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SketchTool {
    Select,
    Line,
    Circle,
    Arc,
    Measure,
}

impl Default for SketchTool { fn default() -> Self { Self::Select } }

impl SketchEditorState {
    pub fn new() -> Self {
        Self {
            active: false,
            node_id: String::new(),
            document: SketchDocument::new(),
            tool: SketchTool::Select,
            selected_entity: None,
            selected_entities: Vec::new(),
            grid_size: 0.5,
            plane: SketchPlane::XY,
            line_start: None,
            drag_from: None,
            drag_offset: glam::DVec2::ZERO,
            drag_start_saved: false,
            show_constraints: true,
            extrude_distance: 2.0,
            extrude_taper_angle: 0.0,
            circle_center: None,
            arc_center: None,
            measure_click_a: None,
            measure_click_b: None,
            measure_result: None,
            editing_constraint_idx: None,
            editing_new_value: 0.0,
            grid_snap: false,
            snap_size: 0.5,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn save_snapshot(&mut self) {
        self.undo_stack.push(self.document.clone());
        self.redo_stack.clear();
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            let current = std::mem::replace(&mut self.document, prev);
            self.redo_stack.push(current);
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            let current = std::mem::replace(&mut self.document, next);
            self.undo_stack.push(current);
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn delete_selected(&mut self) {
        let ids: Vec<EntityId> = self.selected_entities.drain(..).collect();
        if !ids.is_empty() {
            self.save_snapshot();
            for &id in &ids {
                if self.selected_entity == Some(id) {
                    self.selected_entity = None;
                }
                self.document.remove_entity(id);
            }
            self.document.solve().ok();
        } else if let Some(id) = self.selected_entity {
            self.save_snapshot();
            self.document.remove_entity(id);
            self.selected_entity = None;
            self.document.solve().ok();
        }
    }

    pub fn enter(&mut self, node_id: &str, sketch_def: &SketchDef) {
        self.active = true;
        self.node_id = node_id.to_string();
        self.document = SketchDocument::new();
        self.tool = SketchTool::Select;
        self.selected_entity = None;
        self.selected_entities.clear();
        self.plane = sketch_def.plane.clone();
        self.line_start = None;
        self.drag_from = None;
        self.drag_start_saved = false;
        self.circle_center = None;
        self.arc_center = None;
        self.measure_click_a = None;
        self.measure_click_b = None;
        self.measure_result = None;
        self.editing_constraint_idx = None;
        self.undo_stack.clear();
        self.redo_stack.clear();

        for prim in &sketch_def.primitives {
            match *prim {
                SketchPrimitive::Rectangle { x, y, width, height } => {
                    self.document.add_rectangle(x, y, width, height);
                }
                SketchPrimitive::Circle { cx, cy, radius, .. } => {
                    self.document.add_circle(cx, cy, radius);
                }
                SketchPrimitive::Polygon { ref points } => {
                    if points.len() >= 2 {
                        let pids: Vec<EntityId> = points.iter()
                            .map(|p| self.document.add_point(p[0], p[1]))
                            .collect();
                        for i in 0..pids.len() {
                            let next = (i + 1) % pids.len();
                            self.document.add_line(pids[i], pids[next]);
                        }
                    }
                }
                SketchPrimitive::Arc { cx, cy, radius, start_angle, end_angle, .. } => {
                    self.document.add_arc(cx, cy, radius, start_angle, end_angle);
                }
            }
        }
    }

    pub fn exit(&mut self) -> Option<(String, SketchDef)> {
        if !self.active { return None; }
        let node_id = self.node_id.clone();
        let contours = self.document.get_contours();
        let primitives: Vec<SketchPrimitive> = if contours.iter().any(|c| c.len() > 2) {
            contours.into_iter()
                .filter(|c| c.len() > 1)
                .map(|c| SketchPrimitive::Polygon { points: c })
                .collect()
        } else {
            let mut prims = Vec::new();
            for line in &self.document.lines {
                if let (Some(s), Some(e)) = (
                    self.document.points.iter().find(|p| p.id == line.start),
                    self.document.points.iter().find(|p| p.id == line.end),
                ) {
                    prims.push(SketchPrimitive::Polygon {
                        points: vec![[s.x, s.y], [e.x, e.y]],
                    });
                }
            }
            for c in &self.document.circles {
                if let Some(center) = self.document.points.iter().find(|p| p.id == c.center) {
                    prims.push(SketchPrimitive::Circle {
                        cx: center.x, cy: center.y, radius: c.radius, segments: Some(64),
                    });
                }
            }
            prims
        };
        let sketch_def = SketchDef { primitives, plane: self.plane.clone() };
        self.active = false;
        Some((node_id, sketch_def))
    }
}

impl Default for SketchEditorState { fn default() -> Self { Self::new() } }

pub fn sketch_ui(
    mut contexts: EguiContexts,
    mut editor: ResMut<SketchEditorState>,
    mut state: ResMut<AppState>,
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
                    ui.add(egui::DragValue::new(&mut editor.extrude_taper_angle).speed(0.5).suffix("°").range(-60.0..=60.0));
                }
                if ui.button("Extrude").clicked() {
                    let node_id = editor.node_id.clone();
                    let dist = editor.extrude_distance;
                    if let Some((parent_id, _idx)) = find_parent_of(&state.document.recipe.scene, &node_id) {
                        let ext_id = format!("Extrude_{}", node_id);
                        let ext_node = GeometryNode {
                            id: ext_id.clone(),
                            node_type: GeometryNodeType::Extrude(ExtrudeDef {
                                sketch_id: node_id.clone(),
                                distance: dist,
                                direction: None,
                                cap: true,
                                taper_angle: if editor.extrude_taper_angle != 0.0 { Some(editor.extrude_taper_angle) } else { None },
                            }),
                            transform: None,
                            children: vec![],
                            operations: vec![],
                            color: None,
                        };
                        add_child_to(&mut state.document.recipe.scene, &parent_id, ext_node);
                        state.document.evaluate_all();
                        state.mark_dirty();
                        state.document.selection = Some(ext_id);
                    }
                }

                if let Some(ref m) = editor.measure_result {
                    ui.label(m);
                    ui.separator();
                }
                ui.label(entity_label(&editor));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Finish").clicked() {
                        if let Some((node_id, sketch_def)) = editor.exit() {
                            use crate::commands::SetSketchCommand;
                            let cmd = SetSketchCommand {
                                node_id: node_id.clone(),
                                old_sketch: None,
                                new_sketch: sketch_def,
                            };
                            let mut doc = std::mem::take(&mut state.document);
                            state.history.execute(Box::new(cmd), &mut doc);
                            state.document = doc;
                            state.mark_dirty();
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        editor.active = false;
                        editor.document = SketchDocument::new();
                    }
                });
            });
        });

    // Constraint list panel (right side)
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
                editor.document.solve().ok();
                if editor.editing_constraint_idx == Some(idx) {
                    editor.editing_constraint_idx = None;
                }
            }
        });

    // Constraint value editing popup
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
                            editor.document.solve().ok();
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

fn ray_plane_intersection(ray_origin: Vec3, ray_dir: Vec3, plane: &SketchPlane) -> Option<Vec3> {
    let normal = sketch_plane_normal(plane).as_vec3();
    let denom = ray_dir.dot(normal);
    if denom.abs() < 1e-6 { return None; }
    let t = -ray_origin.dot(normal) / denom;
    if t < 0.0 { return None; }
    Some(ray_origin + ray_dir * t)
}

pub fn sketch_input(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<OrbitCamera>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut editor: ResMut<SketchEditorState>,
) {
    if !editor.active { return; }

    let window = windows.single();
    let cursor = match window.cursor_position() {
        Some(c) => c,
        None => return,
    };
    let (cam, cam_transform) = match cameras.get_single() {
        Ok(c) => c,
        Err(_) => return,
    };
    let Ok(ray) = cam.viewport_to_world(cam_transform, cursor) else { return };
    let Some(hit) = ray_plane_intersection(ray.origin, ray.direction.as_vec3(), &editor.plane) else { return };
    let (x, y) = to_2d(hit, &editor.plane);
    // Apply grid snap
    let (x, y) = if editor.grid_snap {
        let s = editor.snap_size;
        ((x / s).round() * s, (y / s).round() * s)
    } else {
        (x, y)
    };

    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if ctrl && keys.just_pressed(KeyCode::KeyZ) {
        if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
            editor.redo();
        } else {
            editor.undo();
        }
        return;
    }
    if ctrl && keys.just_pressed(KeyCode::KeyY) {
        editor.redo();
        return;
    }
    if ctrl && keys.just_pressed(KeyCode::KeyA) {
        let mut all_ids: Vec<EntityId> = editor.document.lines.iter().map(|l| l.id).collect();
        all_ids.extend(editor.document.circles.iter().map(|c| c.id));
        all_ids.extend(editor.document.arcs.iter().map(|a| a.id));
        editor.selected_entities = all_ids;
        return;
    }

    if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
        editor.delete_selected();
        return;
    }

    if mouse.just_pressed(MouseButton::Right) {
        editor.line_start = None;
        editor.drag_from = None;
        editor.drag_start_saved = false;
        editor.circle_center = None;
        editor.arc_center = None;
        editor.measure_click_a = None;
        editor.measure_click_b = None;
        editor.measure_result = None;
        return;
    }

    if mouse.just_pressed(MouseButton::Left) {
        match editor.tool {
            SketchTool::Line => {
                if let Some((sx, sy)) = editor.line_start {
                    editor.save_snapshot();
                    let p1 = editor.document.add_point(sx, sy);
                    let p2 = editor.document.add_point(x, y);
                    let lid = editor.document.add_line(p1, p2);
                    let inferred = editor.document.infer_constraints(
                        lid, glam::DVec2::new(sx, sy), glam::DVec2::new(x, y));
                    for c in inferred {
                        editor.document.add_constraint(c);
                    }
                    editor.document.solve().ok();
                    editor.line_start = None;
                    editor.selected_entity = Some(lid);
                } else {
                    editor.line_start = Some((x, y));
                }
            }
            SketchTool::Select => {
                let point_list: Vec<(EntityId, f64, f64)> = editor.document.points.iter()
                    .map(|p| (p.id, p.x, p.y)).collect();
                if let Some(pid) = pick_point(&point_list, x, y, 0.2) {
                    let pos = editor.document.points.iter().find(|p| p.id == pid).map(|p| (p.x, p.y)).unwrap_or((x, y));
                    editor.drag_from = Some(pid);
                    editor.drag_offset = glam::DVec2::new(x, y) - glam::DVec2::new(pos.0, pos.1);
                    return;
                }
                let mut best = None;
                let mut best_d = 0.15f64;
                for l in &editor.document.lines {
                    if let (Some(s), Some(e)) = (
                        editor.document.points.iter().find(|p| p.id == l.start),
                        editor.document.points.iter().find(|p| p.id == l.end),
                    ) {
                        let sa = glam::DVec2::new(s.x, s.y);
                        let ea = glam::DVec2::new(e.x, e.y);
                        let pa = glam::DVec2::new(x, y);
                        let closest = closest_point_on_line(pa, sa, ea);
                        let d = pa.distance(closest);
                        if d < best_d { best_d = d; best = Some(l.id); }
                    }
                }
                for c in &editor.document.circles {
                    if let Some(center) = editor.document.points.iter().find(|p| p.id == c.center) {
                        let pa = glam::DVec2::new(x, y);
                        let ca = glam::DVec2::new(center.x, center.y);
                        let d_to_perim = (pa.distance(ca) - c.radius).abs();
                        if d_to_perim < best_d { best_d = d_to_perim; best = Some(c.id); }
                    }
                }
                if best.is_none() {
                    // Try picking a constraint for editing
                    let pick: Vec<(usize, (f64, f64))> = editor.document.constraints.iter().enumerate()
                        .map(|(ci, c)| (ci, constraint_marker_pos(c, &editor.document))).collect();
                    for (ci, pos) in &pick {
                        let d = glam::DVec2::new(x - pos.0, y - pos.1).length();
                        if d < 0.15 {
                            let doc = &editor.document;
                            let val = doc.constraints.get(*ci).map(|c| match *c {
                                Constraint::Distance { distance, .. } => distance,
                                Constraint::Angle { angle, .. } => angle,
                                Constraint::Radius { radius, .. } => radius,
                                _ => 0.0,
                            }).unwrap_or(0.0);
                            editor.editing_constraint_idx = Some(*ci);
                            editor.editing_new_value = val;
                            break;
                        }
                    }
                }
                editor.selected_entity = best;
            }
            SketchTool::Circle => {
                if let Some((cx, cy)) = editor.circle_center {
                    let r = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
                    if r > 0.01 {
                        editor.save_snapshot();
                        let id = editor.document.add_circle(cx, cy, r);
                        editor.document.solve().ok();
                        editor.selected_entity = Some(id);
                    }
                    editor.circle_center = None;
                } else {
                    editor.circle_center = Some((x, y));
                }
            }
            SketchTool::Arc => {
                if let Some((cx, cy)) = editor.arc_center {
                    let r = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
                    if r > 0.01 {
                        editor.save_snapshot();
                        let angle = (y - cy).atan2(x - cx);
                        let id = editor.document.add_arc(cx, cy, r, 0.0, angle);
                        editor.document.solve().ok();
                        editor.selected_entity = Some(id);
                    }
                    editor.arc_center = None;
                } else {
                    editor.arc_center = Some((x, y));
                }
            }
            SketchTool::Measure => {
                if let Some((ax, ay)) = editor.measure_click_a {
                    let dx = x - ax;
                    let dy = y - ay;
                    let dist = (dx * dx + dy * dy).sqrt();
                    let angle = dy.atan2(dx).to_degrees();
                    editor.measure_click_b = Some((x, y));
                    editor.measure_result = Some(format!("d: {:.3}  ∠: {:.1}°", dist, angle));
                } else {
                    editor.measure_click_a = Some((x, y));
                    editor.measure_click_b = None;
                    editor.measure_result = None;
                }
            }
        }
    }

    if let Some(pid) = editor.drag_from {
        if mouse.just_released(MouseButton::Left) {
            editor.drag_from = None;
        } else if mouse.pressed(MouseButton::Left) && !editor.drag_start_saved {
            editor.save_snapshot();
            editor.drag_start_saved = true;
            let offset = editor.drag_offset;
            if let Some(p) = editor.document.points.iter_mut().find(|p| p.id == pid) {
                p.x = x - offset.x;
                p.y = y - offset.y;
                editor.document.solve().ok();
            }
        } else if mouse.pressed(MouseButton::Left) {
            let offset = editor.drag_offset;
            if let Some(p) = editor.document.points.iter_mut().find(|p| p.id == pid) {
                p.x = x - offset.x;
                p.y = y - offset.y;
                editor.document.solve().ok();
            }
        }
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

pub use crate::sketch_render::*;

fn find_parent_of<'a>(node: &'a GeometryNode, target: &str) -> Option<(String, usize)> {
    for (i, child) in node.children.iter().enumerate() {
        if child.id == target {
            return Some((node.id.clone(), i));
        }
        if let found @ Some(_) = find_parent_of(child, target) {
            return found;
        }
    }
    None
}

fn add_child_to(node: &mut GeometryNode, parent_id: &str, child: GeometryNode) {
    if node.id == parent_id {
        node.children.push(child);
        return;
    }
    for c in &mut node.children {
        add_child_to(c, parent_id, child.clone());
    }
}
