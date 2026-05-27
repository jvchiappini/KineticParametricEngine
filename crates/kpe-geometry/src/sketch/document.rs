use serde::{Deserialize, Serialize};
use crate::sketch::entities::*;
use crate::sketch::constraints::Constraint;
use crate::sketch::solver::Solver;
use crate::sketch::inference::{InferenceEngine, SnapResult};
use crate::sketch::boolean::extrude_contour_to_3d;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SketchDocument {
    pub points: Vec<Point>,
    pub lines: Vec<Line>,
    pub arcs: Vec<Arc>,
    pub circles: Vec<Circle>,
    pub constraints: Vec<Constraint>,
    next_id: EntityId,
}

impl SketchDocument {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            lines: Vec::new(),
            arcs: Vec::new(),
            circles: Vec::new(),
            constraints: Vec::new(),
            next_id: 1,
        }
    }

    fn alloc_id(&mut self) -> EntityId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn add_point(&mut self, x: f64, y: f64) -> EntityId {
        let id = self.alloc_id();
        self.points.push(Point { id, x, y });
        id
    }

    pub fn add_line(&mut self, start: EntityId, end: EntityId) -> EntityId {
        let id = self.alloc_id();
        self.lines.push(Line { id, start, end });
        id
    }

    pub fn add_line_between(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) -> (EntityId, EntityId, EntityId) {
        let p1 = self.add_point(x1, y1);
        let p2 = self.add_point(x2, y2);
        let line_id = self.add_line(p1, p2);
        (line_id, p1, p2)
    }

    pub fn add_rectangle(&mut self, x: f64, y: f64, w: f64, h: f64) -> Vec<EntityId> {
        let p1 = self.add_point(x, y);
        let p2 = self.add_point(x + w, y);
        let p3 = self.add_point(x + w, y + h);
        let p4 = self.add_point(x, y + h);
        let l1 = self.add_line(p1, p2);
        let l2 = self.add_line(p2, p3);
        let l3 = self.add_line(p3, p4);
        let l4 = self.add_line(p4, p1);
        vec![l1, l2, l3, l4]
    }

    pub fn add_constraint(&mut self, c: Constraint) {
        self.constraints.push(c);
    }

    pub fn solve(&mut self) -> Result<(), String> {
        Solver::solve(&mut self.points, &self.lines, &self.arcs, &self.circles, &self.constraints)
    }

    pub fn snap(&self, x: f64, y: f64, grid_size: f64) -> SnapResult {
        let pos = glam::DVec2::new(x, y);
        let grid = InferenceEngine::snap_to_grid_point(pos, grid_size);

        if let Some(entity_snap) = InferenceEngine::snap_to_entities(pos, &self.points, &self.lines, &self.arcs, &self.circles) {
            return entity_snap;
        }

        SnapResult {
            x: grid.x,
            y: grid.y,
            kind: "grid".into(),
            target_id: None,
        }
    }

    pub fn infer_constraints(&self, line_id: EntityId, start: glam::DVec2, end: glam::DVec2) -> Vec<Constraint> {
        InferenceEngine::infer_constraints(&self.points, &self.lines, line_id, start, end)
    }

    pub fn to_contours(&self) -> Vec<Vec<[f64; 2]>> {
        let mut contours = Vec::new();
        for line in &self.lines {
            if let (Some(s), Some(e)) = (
                self.points.iter().find(|p| p.id == line.start),
                self.points.iter().find(|p| p.id == line.end),
            ) {
                contours.push(vec![
                    [s.x, s.y],
                    [e.x, e.y],
                ]);
            }
        }
        for c in &self.circles {
            if let Some(center) = self.points.iter().find(|p| p.id == c.center) {
                let mut pts = Vec::new();
                let segs = 32;
                for i in 0..segs {
                    let a = (i as f64 / segs as f64) * std::f64::consts::TAU;
                    pts.push([center.x + c.radius * a.cos(), center.y + c.radius * a.sin()]);
                }
                contours.push(pts);
            }
        }
        contours
    }

    pub fn add_circle(&mut self, cx: f64, cy: f64, radius: f64) -> EntityId {
        let center = self.add_point(cx, cy);
        let id = self.alloc_id();
        self.circles.push(Circle { id, center, radius });
        id
    }

    pub fn add_arc(&mut self, cx: f64, cy: f64, radius: f64, start_angle: f64, end_angle: f64) -> EntityId {
        let center = self.add_point(cx, cy);
        let sp = self.add_point(cx + radius * start_angle.cos(), cy + radius * start_angle.sin());
        let ep = self.add_point(cx + radius * end_angle.cos(), cy + radius * end_angle.sin());
        let id = self.alloc_id();
        self.arcs.push(Arc { id, center, start: sp, end: ep, radius, sweep_angle: end_angle - start_angle });
        id
    }

    pub fn remove_entity(&mut self, id: EntityId) {
        self.points.retain(|p| p.id != id);
        self.lines.retain(|l| l.id != id && l.start != id && l.end != id);
        self.arcs.retain(|a| a.id != id && a.center != id && a.start != id && a.end != id);
        self.circles.retain(|c| c.id != id && c.center != id);
    }

    /// Returns ordered open chains then closed loops (like TS walkOrderedChains)
    pub fn get_contours(&self) -> Vec<Vec<[f64; 2]>> {
        use std::collections::{HashMap, HashSet};
        let mut adj: HashMap<EntityId, Vec<EntityId>> = HashMap::new();
        for l in &self.lines {
            adj.entry(l.start).or_default().push(l.end);
            adj.entry(l.end).or_default().push(l.start);
        }
        let mut visited = HashSet::new();
        let mut results: Vec<Vec<[f64; 2]>> = Vec::new();
        let pts = &self.points;

        let mut walk = |start: EntityId, is_loop: bool, vis: &mut HashSet<EntityId>, res: &mut Vec<Vec<[f64; 2]>>| {
            if vis.contains(&start) { return; }
            let mut ordered: Vec<EntityId> = Vec::new();
            let mut cur = Some(start);
            let mut prev: Option<EntityId> = None;
            while let Some(c) = cur {
                if vis.contains(&c) { break; }
                vis.insert(c);
                ordered.push(c);
                let nbs = adj.get(&c).map(|v| v.as_slice()).unwrap_or(&[]);
                let next = nbs.iter().copied().filter(|n| Some(*n) != prev).next();
                if next.is_none() { break; }
                prev = Some(c);
                cur = next;
                if is_loop && cur == Some(start) { break; }
            }
            let out: Vec<[f64; 2]> = ordered.iter().filter_map(|id| pts.iter().find(|p| p.id == *id).map(|p| [p.x, p.y])).collect();
            if out.len() >= 2 { res.push(out); }
        };

        for (&pid, nbs) in &adj { if nbs.len() == 1 { walk(pid, false, &mut visited, &mut results); } }
        for (&pid, _) in &adj { if !visited.contains(&pid) { walk(pid, true, &mut visited, &mut results); } }
        results
    }

    /// Count degrees of freedom: free points × 2 + circles × 1 + arcs × 3 − constraints
    pub fn count_dof(&self) -> u32 {
        let mut dof = 0u32;
        for p in &self.points {
            let is_fixed = self.constraints.iter().any(|c| matches!(c, Constraint::Fix { point, .. } if *point == p.id));
            if !is_fixed { dof += 2; }
        }
        for _c in &self.circles { dof += 1; }
        for _a in &self.arcs { dof += 3; }
        for c in &self.constraints {
            dof -= match c {
                Constraint::Coincident { .. } => 2,
                Constraint::Horizontal { .. } | Constraint::Vertical { .. }
                | Constraint::Parallel { .. } | Constraint::Perpendicular { .. }
                | Constraint::EqualLength { .. } | Constraint::Tangent { .. }
                | Constraint::Distance { .. } | Constraint::Angle { .. }
                | Constraint::Collinear { .. } => 1,
                Constraint::Fix { .. } => 0,
                Constraint::Midpoint { .. } => 1,
                Constraint::Radius { .. } => 1,
            };
        }
        dof.max(0)
    }

    pub fn extrude_contours(&self, distance: f64) -> (Vec<[f64; 3]>, Vec<[u32; 3]>) {
        let mut all_verts = Vec::new();
        let mut all_tris = Vec::new();
        for line in &self.lines {
            if let (Some(s), Some(e)) = (
                self.points.iter().find(|p| p.id == line.start),
                self.points.iter().find(|p| p.id == line.end),
            ) {
                let contour = vec![glam::DVec2::new(s.x, s.y), glam::DVec2::new(e.x, e.y), glam::DVec2::new(s.x, s.y)];
                let (verts, tris) = extrude_contour_to_3d(&contour, distance, false, false);
                let base = all_verts.len() as u32;
                all_verts.extend(verts);
                for t in tris {
                    all_tris.push([t[0] + base, t[1] + base, t[2] + base]);
                }
            }
        }
        (all_verts, all_tris)
    }
}

impl Default for SketchDocument {
    fn default() -> Self {
        Self::new()
    }
}
