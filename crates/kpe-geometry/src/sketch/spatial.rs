use std::collections::HashMap;
use glam::DVec2;
use crate::sketch::entities::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpatialEntity {
    Point(EntityId),
    Line(EntityId),
    Circle(EntityId),
    Arc(EntityId),
}

/// Grid-based spatial index for fast proximity queries.
///
/// Each cell stores references to entities whose bounding boxes
/// overlap that cell.  Query by position + radius returns candidate
/// entities for fine-grained distance checks.
pub struct SpatialIndex {
    cell_size: f64,
    cells: HashMap<(i32, i32), Vec<SpatialEntity>>,
}

impl SpatialIndex {
    pub fn new(cell_size: f64) -> Self {
        Self { cell_size, cells: HashMap::new() }
    }

    fn cell(&self, x: f64, y: f64) -> (i32, i32) {
        ((x / self.cell_size).floor() as i32, (y / self.cell_size).floor() as i32)
    }

    fn cells_in_aabb(&self, min: DVec2, max: DVec2) -> Vec<(i32, i32)> {
        let cmin = self.cell(min.x, min.y);
        let cmax = self.cell(max.x, max.y);
        let mut out = Vec::new();
        for cy in cmin.1..=cmax.1 {
            for cx in cmin.0..=cmax.0 {
                out.push((cx, cy));
            }
        }
        out
    }

    fn insert(&mut self, cell: (i32, i32), entity: SpatialEntity) {
        self.cells.entry(cell).or_default().push(entity);
    }

    pub fn add_point(&mut self, pt: &Point) {
        let cell = self.cell(pt.x, pt.y);
        self.insert(cell, SpatialEntity::Point(pt.id));
    }

    pub fn add_line(&mut self, line: &Line, points: &[Point]) {
        let s = points.iter().find(|p| p.id == line.start).map(|p| DVec2::new(p.x, p.y)).unwrap_or(DVec2::ZERO);
        let e = points.iter().find(|p| p.id == line.end).map(|p| DVec2::new(p.x, p.y)).unwrap_or(DVec2::ZERO);
        let min = DVec2::new(s.x.min(e.x), s.y.min(e.y));
        let max = DVec2::new(s.x.max(e.x), s.y.max(e.y));
        for c in self.cells_in_aabb(min, max) {
            self.insert(c, SpatialEntity::Line(line.id));
        }
    }

    pub fn add_circle(&mut self, circle: &Circle, points: &[Point]) {
        if let Some(center) = points.iter().find(|p| p.id == circle.center) {
            let r = circle.radius;
            let min = DVec2::new(center.x - r, center.y - r);
            let max = DVec2::new(center.x + r, center.y + r);
            for c in self.cells_in_aabb(min, max) {
                self.insert(c, SpatialEntity::Circle(circle.id));
            }
        }
    }

    pub fn add_arc(&mut self, arc: &Arc, points: &[Point]) {
        if let Some(center) = points.iter().find(|p| p.id == arc.center) {
            let r = arc.radius;
            let min = DVec2::new(center.x - r, center.y - r);
            let max = DVec2::new(center.x + r, center.y + r);
            for c in self.cells_in_aabb(min, max) {
                self.insert(c, SpatialEntity::Arc(arc.id));
            }
        }
    }

    /// Build an index from sketch entity lists.
    pub fn build(points: &[Point], lines: &[Line], arcs: &[Arc], circles: &[Circle], cell_size: f64) -> Self {
        let mut idx = Self::new(cell_size);
        for p in points { idx.add_point(p); }
        for l in lines { idx.add_line(l, points); }
        for a in arcs { idx.add_arc(a, points); }
        for c in circles { idx.add_circle(c, points); }
        idx
    }

    /// Return all entity references whose cell may overlap a query circle.
    /// Caller must perform exact distance checks on the candidates.
    pub fn query(&self, x: f64, y: f64, radius: f64) -> Vec<SpatialEntity> {
        let min = DVec2::new(x - radius, y - radius);
        let max = DVec2::new(x + radius, y + radius);
        let mut seen = Vec::new();
        let mut dedup: Vec<EntityId> = Vec::new();
        for cell in self.cells_in_aabb(min, max) {
            if let Some(entries) = self.cells.get(&cell) {
                for e in entries {
                    let id = match e { SpatialEntity::Point(id) | SpatialEntity::Line(id) | SpatialEntity::Circle(id) | SpatialEntity::Arc(id) => *id };
                    if !dedup.contains(&id) {
                        dedup.push(id);
                        seen.push(*e);
                    }
                }
            }
        }
        seen
    }

    /// Convenience: find the nearest point within `radius`.
    pub fn nearest_point(&self, x: f64, y: f64, radius: f64, points: &[Point]) -> Option<(EntityId, f64)> {
        let candidates = self.query(x, y, radius);
        let mut best: Option<(EntityId, f64)> = None;
        for e in &candidates {
            if let SpatialEntity::Point(pid) = e {
                if let Some(pt) = points.iter().find(|p| p.id == *pid) {
                    let d = DVec2::new(pt.x - x, pt.y - y).length();
                    if d <= radius && best.map_or(true, |(_, b)| d < b) {
                        best = Some((*pid, d));
                    }
                }
            }
        }
        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nearest_point() {
        let points = vec![Point { id: 1, x: 0.0, y: 0.0 }, Point { id: 2, x: 1.0, y: 1.0 }];
        let lines = vec![];
        let arcs = vec![];
        let circles = vec![];
        let idx = SpatialIndex::build(&points, &lines, &arcs, &circles, 0.5);
        let result = idx.nearest_point(0.1, 0.1, 0.5, &points);
        assert_eq!(result, Some((1, DVec2::new(0.1, 0.1).length())));
    }

    #[test]
    fn test_no_point_in_range() {
        let points = vec![Point { id: 1, x: 10.0, y: 10.0 }];
        let idx = SpatialIndex::build(&points, &[], &[], &[], 0.5);
        assert!(idx.nearest_point(0.0, 0.0, 1.0, &points).is_none());
    }
}
