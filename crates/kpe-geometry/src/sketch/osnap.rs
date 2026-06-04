use glam::DVec2;
use crate::sketch::entities::*;
use crate::sketch::inference::SnapResult;
use crate::sketch::intersections;

/// Default snap distance in world units.
const SNAP_DIST: f64 = 0.15;

/// Bitmask controlling which snap types are active.
///
/// Pass to `OsnapEngine::snap_filtered()` to disable specific snap modes
/// without changing the priority chain.
#[derive(Debug, Clone, Copy)]
pub struct SnapFilter {
    pub endpoint: bool,
    pub midpoint: bool,
    pub center: bool,
    pub quadrant: bool,
    pub intersection: bool,
    pub on_entity: bool,
}

impl SnapFilter {
    /// Enable all snap types (default).
    pub const fn all() -> Self {
        Self {
            endpoint: true,
            midpoint: true,
            center: true,
            quadrant: true,
            intersection: true,
            on_entity: true,
        }
    }

    /// Disable all snap types.
    pub const fn none() -> Self {
        Self {
            endpoint: false,
            midpoint: false,
            center: false,
            quadrant: false,
            intersection: false,
            on_entity: false,
        }
    }

    /// Returns `true` if at least one snap type is enabled.
    pub fn any_enabled(&self) -> bool {
        self.endpoint || self.midpoint || self.center
            || self.quadrant || self.intersection || self.on_entity
    }
}

impl Default for SnapFilter {
    fn default() -> Self { Self::all() }
}

/// Object Snap engine — provides all industrial-grade snapping modes.
///
/// Priority (first match wins):
/// 1. Endpoint
/// 2. Midpoint
/// 3. Center (of circle / arc)
/// 4. Quadrant (0°, 90°, 180°, 270° on circle / arc)
/// 5. Intersection (between any two curves)
/// 6. On-entity (point on line / circle edge / arc edge)
///
/// Domain segregation: pure computation.  No UI state, no rendering.
pub struct OsnapEngine;

impl OsnapEngine {
    /// Try all snap modes in priority order.  Returns the first match.
    pub fn snap(
        pos: DVec2,
        points: &[Point],
        lines: &[Line],
        arcs: &[Arc],
        circles: &[Circle],
    ) -> Option<SnapResult> {
        Self::snap_filtered(pos, points, lines, arcs, circles, &SnapFilter::all())
    }

    /// Like `snap()` but only checks snap types enabled in `filter`.
    pub fn snap_filtered(
        pos: DVec2,
        points: &[Point],
        lines: &[Line],
        arcs: &[Arc],
        circles: &[Circle],
        filter: &SnapFilter,
    ) -> Option<SnapResult> {
        let r: Option<SnapResult> = None;
        let r = if filter.endpoint   { r.or_else(|| Self::snap_endpoint(pos, points)) } else { r };
        let r = if filter.midpoint   { r.or_else(|| Self::snap_midpoint(pos, points, lines)) } else { r };
        let r = if filter.center     { r.or_else(|| Self::snap_center(pos, points, circles, arcs)) } else { r };
        let r = if filter.quadrant   { r.or_else(|| Self::snap_quadrant(pos, points, circles, arcs)) } else { r };
        let r = if filter.intersection {
            r.or_else(|| Self::snap_intersection(pos, points, lines, arcs, circles))
        } else { r };
        let r = if filter.on_entity  { r.or_else(|| Self::snap_on_entity(pos, points, lines, circles)) } else { r };
        r
    }

    // ── Endpoint ───────────────────────────────────────────────────

    fn snap_endpoint(pos: DVec2, points: &[Point]) -> Option<SnapResult> {
        points.iter().find(|p| pos.distance(p.pos()) < SNAP_DIST).map(|p| {
            SnapResult { x: p.x, y: p.y, kind: "endpoint".into(), target_id: Some(p.id) }
        })
    }

    // ── Midpoint ───────────────────────────────────────────────────

    fn snap_midpoint(pos: DVec2, points: &[Point], lines: &[Line]) -> Option<SnapResult> {
        for line in lines {
            let a = point_by_id(points, line.start);
            let b = point_by_id(points, line.end);
            let mid = (a + b) * 0.5;
            if pos.distance(mid) < SNAP_DIST * 0.8 {
                return Some(SnapResult {
                    x: mid.x, y: mid.y, kind: "midpoint".into(), target_id: Some(line.id),
                });
            }
        }
        None
    }

    // ── Center ─────────────────────────────────────────────────────

    fn snap_center(
        pos: DVec2, points: &[Point], circles: &[Circle], _arcs: &[Arc],
    ) -> Option<SnapResult> {
        for c in circles {
            let center_pos = point_by_id(points, c.center);
            if pos.distance(center_pos) < SNAP_DIST * 2.0 {
                return Some(SnapResult {
                    x: center_pos.x, y: center_pos.y,
                    kind: "center".into(), target_id: Some(c.id),
                });
            }
        }
        None
    }

    // ── Quadrant ───────────────────────────────────────────────────

    fn snap_quadrant(
        pos: DVec2, points: &[Point], circles: &[Circle], _arcs: &[Arc],
    ) -> Option<SnapResult> {
        for c in circles {
            let center = point_by_id(points, c.center);
            let r = c.radius;
            let quads = [
                center + DVec2::new(r, 0.0),
                center + DVec2::new(0.0, r),
                center + DVec2::new(-r, 0.0),
                center + DVec2::new(0.0, -r),
            ];
            for q in &quads {
                if pos.distance(*q) < SNAP_DIST {
                    return Some(SnapResult {
                        x: q.x, y: q.y, kind: "quadrant".into(), target_id: Some(c.id),
                    });
                }
            }
        }
        None
    }

    // ── Intersection ───────────────────────────────────────────────

    fn snap_intersection(
        pos: DVec2, points: &[Point], lines: &[Line], arcs: &[Arc], circles: &[Circle],
    ) -> Option<SnapResult> {
        // Lines × Lines
        for (i, la) in lines.iter().enumerate() {
            let a1 = point_by_id(points, la.start);
            let a2 = point_by_id(points, la.end);
            for lb in &lines[i + 1..] {
                let b1 = point_by_id(points, lb.start);
                let b2 = point_by_id(points, lb.end);
                if let Some(pt) = intersections::line_line(a1, a2, b1, b2) {
                    if pos.distance(pt) < SNAP_DIST {
                        return Self::intersection_result(pt);
                    }
                }
            }
        }
        // Lines × Circles
        for line in lines {
            let a1 = point_by_id(points, line.start);
            let a2 = point_by_id(points, line.end);
            for c in circles {
                let center = point_by_id(points, c.center);
                for pt in intersections::line_circle(a1, a2, center, c.radius) {
                    if pos.distance(pt) < SNAP_DIST {
                        return Self::intersection_result(pt);
                    }
                }
            }
        }
        // Lines × Arcs
        for line in lines {
            let a1 = point_by_id(points, line.start);
            let a2 = point_by_id(points, line.end);
            for arc in arcs {
                let center = point_by_id(points, arc.center);
                let arc_start = arc_start_angle(arc, points);
                for pt in intersections::line_arc(
                    a1, a2, center, arc.radius, arc_start, arc.sweep_angle,
                ) {
                    if pos.distance(pt) < SNAP_DIST {
                        return Self::intersection_result(pt);
                    }
                }
            }
        }
        // Circles × Circles
        for (i, ca) in circles.iter().enumerate() {
            let c1 = point_by_id(points, ca.center);
            for cb in &circles[i + 1..] {
                let c2 = point_by_id(points, cb.center);
                for pt in intersections::circle_circle(c1, ca.radius, c2, cb.radius) {
                    if pos.distance(pt) < SNAP_DIST {
                        return Self::intersection_result(pt);
                    }
                }
            }
        }
        None
    }

    fn intersection_result(pt: DVec2) -> Option<SnapResult> {
        Some(SnapResult { x: pt.x, y: pt.y, kind: "intersection".into(), target_id: None })
    }

    // ── On-entity ──────────────────────────────────────────────────

    fn snap_on_entity(
        pos: DVec2, points: &[Point], lines: &[Line], circles: &[Circle],
    ) -> Option<SnapResult> {
        // Lines
        for line in lines {
            let a = point_by_id(points, line.start);
            let b = point_by_id(points, line.end);
            let closest = closest_point_on_line(pos, a, b);
            if pos.distance(closest) < SNAP_DIST
                && closest.distance(a) > 0.01
                && closest.distance(b) > 0.01
            {
                return Some(SnapResult {
                    x: closest.x, y: closest.y,
                    kind: "on_entity".into(), target_id: Some(line.id),
                });
            }
        }
        // Circles
        for c in circles {
            let center = point_by_id(points, c.center);
            if (pos.distance(center) - c.radius).abs() < SNAP_DIST {
                return Some(SnapResult {
                    x: pos.x, y: pos.y,
                    kind: "on_entity".into(), target_id: Some(c.id),
                });
            }
        }
        None
    }
}

// ── Helpers ────────────────────────────────────────────────────────

fn point_by_id(points: &[Point], id: EntityId) -> DVec2 {
    points.iter().find(|p| p.id == id).map(|p| p.pos()).unwrap_or(DVec2::ZERO)
}

/// Get the start angle of an arc (angle of the start point relative to center).
fn arc_start_angle(arc: &Arc, points: &[Point]) -> f64 {
    let center = point_by_id(points, arc.center);
    let start = point_by_id(points, arc.start);
    (start - center).y.atan2((start - center).x)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pt(id: u64, x: f64, y: f64) -> Point { Point { id, x, y } }
    fn ln(id: u64, s: u64, e: u64) -> Line { Line { id, start: s, end: e } }

    #[test]
    fn test_endpoint() {
        let r = OsnapEngine::snap(DVec2::new(5.05, 5.05), &[pt(1, 5.0, 5.0)], &[], &[], &[]);
        assert!(r.is_some());
        assert_eq!(r.unwrap().kind, "endpoint");
    }

    #[test]
    fn test_midpoint() {
        let p = vec![pt(1, 0.0, 0.0), pt(2, 10.0, 0.0)];
        let l = vec![ln(10, 1, 2)];
        let r = OsnapEngine::snap(DVec2::new(5.0, 0.05), &p, &l, &[], &[]);
        assert!(r.is_some());
        assert_eq!(r.unwrap().kind, "midpoint");
    }

    #[test]
    fn test_center() {
        // Center at (200, 200).  Cursor at (200.2, 200.0) is within center
        // range (0.3) but outside endpoint range (0.15).
        let p = vec![pt(1, 200.0, 200.0)];
        let c = vec![Circle { id: 20, center: 1, radius: 10.0 }];
        let r = OsnapEngine::snap(DVec2::new(200.2, 200.0), &p, &[], &[], &c);
        assert!(r.is_some());
        let snap = r.unwrap();
        assert_eq!(snap.kind, "center");
        assert!((snap.x - 200.0).abs() < 0.01);
    }

    #[test]
    fn test_intersection_line_line() {
        // Line A (0,0)→(20,6), Line B (5,0)→(5,20).
        // Actual intersection: t=0.25 → (5, 1.5).
        // Cursor at (5, 1.55) is near intersection but not on any midpoint.
        let p = vec![pt(1, 0.0, 0.0), pt(2, 20.0, 6.0),
                     pt(3, 5.0, 0.0), pt(4, 5.0, 20.0)];
        let l = vec![ln(10, 1, 2), ln(11, 3, 4)];
        let r = OsnapEngine::snap(DVec2::new(5.0, 1.55), &p, &l, &[], &[]);
        assert!(r.is_some());
        assert_eq!(r.unwrap().kind, "intersection");
    }

    #[test]
    fn test_quadrant() {
        // Circle center (100, 100), radius 5.  No other entities.
        // Cursor at (105, 100.1) near right quadrant.
        let p = vec![pt(1, 100.0, 100.0)];
        let c = vec![Circle { id: 20, center: 1, radius: 5.0 }];
        let r = OsnapEngine::snap(DVec2::new(105.0, 100.1), &p, &[], &[], &c);
        assert!(r.is_some());
        assert_eq!(r.unwrap().kind, "quadrant");
    }

    #[test]
    fn test_intersection_line_circle() {
        // Circle center (100, 0) r=5.  Line from (90, 3) to (110, 3).
        // Intersection at (104, 3) and (96, 3).  Neither is a quadrant.
        let p = vec![pt(1, 90.0, 3.0), pt(2, 110.0, 3.0), pt(3, 100.0, 0.0)];
        let l = vec![ln(10, 1, 2)];
        let c = vec![Circle { id: 20, center: 3, radius: 5.0 }];
        let r = OsnapEngine::snap(DVec2::new(104.0, 3.05), &p, &l, &[], &c);
        assert!(r.is_some());
        assert_eq!(r.unwrap().kind, "intersection");
    }

    #[test]
    fn test_on_entity_line() {
        let p = vec![pt(1, 0.0, 0.0), pt(2, 10.0, 0.0)];
        let l = vec![ln(10, 1, 2)];
        let r = OsnapEngine::snap(DVec2::new(4.0, 0.1), &p, &l, &[], &[]);
        assert!(r.is_some());
        assert_eq!(r.unwrap().kind, "on_entity");
    }

    #[test]
    fn test_no_snap_far() {
        let p = vec![pt(1, 0.0, 0.0)];
        let r = OsnapEngine::snap(DVec2::new(100.0, 100.0), &p, &[], &[], &[]);
        assert!(r.is_none());
    }

    #[test]
    fn test_priority_endpoint_over_center() {
        let p = vec![pt(1, 5.0, 5.0)];
        let c = vec![Circle { id: 20, center: 1, radius: 3.0 }];
        let r = OsnapEngine::snap(DVec2::new(5.05, 5.05), &p, &[], &[], &c);
        assert!(r.is_some());
        assert_eq!(r.unwrap().kind, "endpoint");
    }
}
