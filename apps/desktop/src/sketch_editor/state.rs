use bevy::prelude::*;
use kpe_geometry::sketch::document::SketchDocument;
use kpe_geometry::sketch::entities::EntityId;
use kpe_geometry::sketch::constraints::Constraint;
use kpe_schema::geometry::{SketchDef, SketchPlane, SketchPrimitive};
use super::solver;

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
    pub last_solve_error: Option<String>,
    pub dof_status: Vec<(bool, bool)>,
    pub pending_extrude: Option<PendingExtrude>,
    pub pending_finish: bool,
    pub pending_cancel: bool,
    undo_stack: Vec<SketchDocument>,
    redo_stack: Vec<SketchDocument>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EntityKind {
    Point(EntityId),
    Line(EntityId),
    Circle(EntityId),
    Arc(EntityId),
}

#[derive(Clone)]
pub struct PendingExtrude {
    pub distance: f64,
    pub taper_angle: f64,
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
            last_solve_error: None,
            pending_extrude: None,
            pending_finish: false,
            pending_cancel: false,
            dof_status: Vec::new(),
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

    pub fn selected_ids(&self) -> Vec<EntityId> {
        let mut ids = self.selected_entities.clone();
        if let Some(single) = self.selected_entity {
            if !ids.contains(&single) {
                ids.push(single);
            }
        }
        ids
    }

    pub fn inject_constraint(&mut self, c: Constraint) {
        self.save_snapshot();
        self.document.constraints.push(c);
        solver::solve_sync(self);
    }

    pub fn find_in_selection(&self, id: EntityId) -> Option<EntityKind> {
        if self.document.points.iter().any(|p| p.id == id) {
            return Some(EntityKind::Point(id));
        }
        if self.document.lines.iter().any(|l| l.id == id) {
            return Some(EntityKind::Line(id));
        }
        if self.document.circles.iter().any(|c| c.id == id) {
            return Some(EntityKind::Circle(id));
        }
        if self.document.arcs.iter().any(|a| a.id == id) {
            return Some(EntityKind::Arc(id));
        }
        None
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
            solver::solve_sync(self);
        } else if let Some(id) = self.selected_entity {
            self.save_snapshot();
            self.document.remove_entity(id);
            self.selected_entity = None;
            solver::solve_sync(self);
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
        self.last_solve_error = None;
        self.pending_extrude = None;
        self.pending_finish = false;
        self.pending_cancel = false;
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
        let sketch_def = SketchDef { primitives, plane: self.plane.clone(), extrude: None };
        self.active = false;
        Some((node_id, sketch_def))
    }
}

impl Default for SketchEditorState { fn default() -> Self { Self::new() } }

pub fn entity_label(editor: &SketchEditorState) -> String {
    format!("{}L {}C {}A {}P {}Cst",
        editor.document.lines.len(),
        editor.document.circles.len(),
        editor.document.arcs.len(),
        editor.document.points.len(),
        editor.document.constraints.len(),
    )
}
