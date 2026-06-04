use glam::DVec2;

/// Default polar tracking angles in degrees (every 45°).
const DEFAULT_ANGLES_DEG: &[f64] = &[0.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0];

/// Default snap tolerance in degrees.
const DEFAULT_TOLERANCE_DEG: f64 = 3.0;

/// Result of a successful polar snap.
#[derive(Debug, Clone, Copy)]
pub struct PolarSnapResult {
    /// The snapped cursor position in world space.
    pub snapped_point: DVec2,
    /// The snapped angle in degrees (one of the canonical snap angles).
    pub angle_deg: f64,
    /// The distance from `from` to `snapped_point`.
    pub distance: f64,
}

/// Magnetic polar-angle tracker for precision drafting.
///
/// When the user draws a line, `PolarSnap` checks whether the cursor angle
/// falls within `tolerance` of any canonical angle (0°, 45°, 90°, etc.).
/// If so, it snaps the cursor to the nearest position along that ray.
///
/// # Domain segregation
///
/// `PolarSnap` is pure computational geometry.  It knows nothing about the
/// UI, keyboard events, or rendering.  The desktop app calls `snap()` in
/// its input event loop and uses the result to override the free-form
/// cursor position.
pub struct PolarSnap {
    /// Canonical snap angles in radians (pre-computed from degrees).
    snap_angles_rad: Vec<f64>,
    /// Tolerance in radians.
    tolerance_rad: f64,
}

impl PolarSnap {
    /// Create a new `PolarSnap` with default angles (every 45°) and the
    /// given tolerance in degrees.
    pub fn new(tolerance_deg: f64) -> Self {
        Self::with_angles(DEFAULT_ANGLES_DEG, tolerance_deg)
    }

    /// Create a `PolarSnap` with a custom set of snap angles.
    ///
    /// `angles_deg` — the canonical angles in degrees (e.g. `&[0.0, 30.0,
    /// 45.0, 90.0]`).
    /// `tolerance_deg` — how close the cursor must be (in degrees) to
    /// trigger a snap.
    pub fn with_angles(angles_deg: &[f64], tolerance_deg: f64) -> Self {
        let snap_angles_rad: Vec<f64> = angles_deg
            .iter()
            .map(|&d| d.to_radians())
            .collect();
        let tolerance_rad = tolerance_deg.to_radians();
        Self {
            snap_angles_rad,
            tolerance_rad,
        }
    }

    /// Return the default snap angles in degrees (every 45°).
    pub fn default_angles_deg() -> &'static [f64] {
        DEFAULT_ANGLES_DEG
    }

    /// Try to snap the cursor to a canonical polar angle.
    ///
    /// Given a line defined by `from → to`, compute the angle of the
    /// segment and check it against every canonical angle.  If the
    /// difference is within `tolerance`, the cursor is perpendicularly
    /// projected onto the nearest canonical ray (so the dominant-axis
    /// distance is preserved).
    ///
    /// Returns `None` when no canonical angle is close enough.
    pub fn snap(&self, from: DVec2, to: DVec2) -> Option<PolarSnapResult> {
        let delta = to - from;
        let cursor_angle = delta.y.atan2(delta.x);
        let distance = delta.length();

        if distance < 1e-12 {
            return None;
        }

        let mut best: Option<(f64, f64)> = None; // (angle_rad, diff)

        for &snap_rad in &self.snap_angles_rad {
            let mut diff = (cursor_angle - snap_rad).abs();
            if diff > std::f64::consts::PI {
                diff = std::f64::consts::TAU - diff;
            }
            if diff <= self.tolerance_rad {
                let is_better = best.map_or(true, |(_, best_diff)| diff < best_diff);
                if is_better {
                    best = Some((snap_rad, diff));
                }
            }
        }

        best.map(|(snap_rad, _)| {
            // Project delta perpendicularly onto the snap direction.
            let dir = DVec2::new(snap_rad.cos(), snap_rad.sin());
            let proj_dist = delta.dot(dir);
            let snapped_point = from + dir * proj_dist;
            PolarSnapResult {
                snapped_point,
                angle_deg: snap_rad.to_degrees(),
                distance: proj_dist.abs(),
            }
        })
    }
}

impl Default for PolarSnap {
    fn default() -> Self {
        Self::new(DEFAULT_TOLERANCE_DEG)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_approx_eq(a: DVec2, b: DVec2, eps: f64) {
        assert!(a.distance(b) < eps, "left: {:?}, right: {:?}", a, b);
    }

    #[test]
    fn test_snap_horizontal_right() {
        let snap = PolarSnap::default();
        let from = DVec2::new(0.0, 0.0);
        let to = DVec2::new(10.0, 0.5);
        let result = snap.snap(from, to);
        assert!(result.is_some());
        let r = result.unwrap();
        assert!((r.angle_deg - 0.0).abs() < 1.0);
        // Perpendicular projection: X preserved, Y zeroed.
        assert_approx_eq(r.snapped_point, DVec2::new(10.0, 0.0), 1e-9);
    }

    #[test]
    fn test_snap_vertical_up() {
        let snap = PolarSnap::default();
        let from = DVec2::new(0.0, 0.0);
        let to = DVec2::new(0.3, 8.0);
        let result = snap.snap(from, to);
        assert!(result.is_some());
        let r = result.unwrap();
        assert!((r.angle_deg - 90.0).abs() < 1.0);
        // Perpendicular projection: Y preserved, X zeroed.
        assert_approx_eq(r.snapped_point, DVec2::new(0.0, 8.0), 1e-9);
    }

    #[test]
    fn test_snap_diagonal_45() {
        let snap = PolarSnap::default();
        let from = DVec2::new(0.0, 0.0);
        let to = DVec2::new(7.0, 6.5);
        let result = snap.snap(from, to);
        assert!(result.is_some());
        let r = result.unwrap();
        assert!((r.angle_deg - 45.0).abs() < 1.0);
        // Perpendicular projection onto 45° ray.
        let dir = DVec2::new(1.0, 1.0).normalize();
        let proj_dist = to.dot(dir);
        let expected = dir * proj_dist;
        assert_approx_eq(r.snapped_point, expected, 1e-9);
    }

    #[test]
    fn test_no_snap_when_outside_tolerance() {
        let snap = PolarSnap::new(1.0); // very tight tolerance
        let from = DVec2::new(0.0, 0.0);
        let to = DVec2::new(10.0, 1.0); // ~5.7° — outside 1° tolerance
        assert!(snap.snap(from, to).is_none());
    }

    #[test]
    fn test_snap_negative_angle() {
        let snap = PolarSnap::default();
        let from = DVec2::new(0.0, 0.0);
        let to = DVec2::new(5.0, -0.2); // slightly below horizontal
        let result = snap.snap(from, to);
        assert!(result.is_some());
        // Should snap to 0° (or 360° / 0°), not to a negative angle.
        let r = result.unwrap();
        assert!((r.angle_deg - 0.0).abs() < 1.0
            || (r.angle_deg - 360.0).abs() < 1.0);
    }

    #[test]
    fn test_custom_angles_30_deg() {
        let snap = PolarSnap::with_angles(&[30.0, 60.0], 2.0);
        let from = DVec2::new(0.0, 0.0);
        // At ~31° — should snap to 30°
        let to = DVec2::new(10.0, 6.0);
        let result = snap.snap(from, to);
        assert!(result.is_some());
        let r = result.unwrap();
        assert!((r.angle_deg - 30.0).abs() < 1.0);
    }

    #[test]
    fn test_zero_length_no_snap() {
        let snap = PolarSnap::default();
        let from = DVec2::new(5.0, 5.0);
        assert!(snap.snap(from, from).is_none());
    }

    #[test]
    fn test_default_angles_include_all_octants() {
        let angles = PolarSnap::default_angles_deg();
        assert_eq!(angles.len(), 8);
        // Verify 45° increments.
        for (i, &a) in angles.iter().enumerate() {
            assert!((a - (i as f64 * 45.0)).abs() < 1e-9);
        }
    }
}
