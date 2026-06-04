use glam::DVec2;
use crate::sketch::polar_tracking::PolarSnap;

/// Captures explicit numeric overrides entered by the user during a drawing
/// operation (e.g. "50" for distance, "45" for angle).
///
/// When the user types a number while drawing a line, the distance (or angle)
/// is locked to that value.  The `resolve` method combines the override with
/// the free-form mouse position and polar snapping to produce the final
/// placement point.
///
/// # Domain segregation
///
/// `DynamicInput` is pure computation.  It holds only the numeric override
/// values.  The UI layer is responsible for intercepting keyboard input and
/// calling `set_distance` / `set_angle`.
#[derive(Debug, Clone)]
pub struct DynamicInput {
    /// Explicit distance override (mm).  `None` → use the raw cursor distance.
    pub distance: Option<f64>,
    /// Explicit angle override (degrees).  `None` → use polar snap or free angle.
    pub angle_deg: Option<f64>,
}

impl DynamicInput {
    /// Create a new `DynamicInput` with no overrides.
    pub fn new() -> Self {
        Self {
            distance: None,
            angle_deg: None,
        }
    }

    /// Set the distance override (e.g. from keyboard entry "120").
    pub fn set_distance(&mut self, value: f64) {
        self.distance = Some(value.abs());
    }

    /// Set the angle override in degrees.
    pub fn set_angle_deg(&mut self, value_deg: f64) {
        self.angle_deg = Some(value_deg % 360.0);
    }

    /// Clear all overrides (e.g. after the drawing operation completes).
    pub fn clear(&mut self) {
        self.distance = None;
        self.angle_deg = None;
    }

    /// Returns `true` if any override is active.
    pub fn is_active(&self) -> bool {
        self.distance.is_some() || self.angle_deg.is_some()
    }

    /// Resolve the final endpoint given the origin, current mouse position,
    /// and an optional `PolarSnap` engine.
    ///
    /// Priority for angle:
    /// 1. Explicit `angle_deg` override (if set)
    /// 2. Polar snap result (if the cursor is near a canonical angle)
    /// 3. Free-form angle from `origin → mouse_pos`
    ///
    /// Priority for distance:
    /// 1. Explicit `distance` override (if set)
    /// 2. Raw distance from `origin → mouse_pos`
    pub fn resolve(
        &self,
        origin: DVec2,
        mouse_pos: DVec2,
        polar_snap: Option<&PolarSnap>,
    ) -> DynamicResult {
        let raw_delta = mouse_pos - origin;
        let raw_distance = raw_delta.length();
        let raw_angle_rad = raw_delta.y.atan2(raw_delta.x);

        // Determine angle.
        let (angle_rad, snapped) = if let Some(deg) = self.angle_deg {
            (deg.to_radians(), false)
        } else if let Some(snap) = polar_snap.and_then(|ps| ps.snap(origin, mouse_pos)) {
            (snap.angle_deg.to_radians(), true)
        } else {
            (raw_angle_rad, false)
        };

        // Determine distance.
        let distance = self.distance.unwrap_or(raw_distance);

        // Compute final point.
        let final_point = origin + DVec2::new(angle_rad.cos(), angle_rad.sin()) * distance;

        DynamicResult {
            point: final_point,
            distance,
            angle_deg: angle_rad.to_degrees() % 360.0,
            snapped,
            distance_overridden: self.distance.is_some(),
            angle_overridden: self.angle_deg.is_some(),
        }
    }
}

impl Default for DynamicInput {
    fn default() -> Self {
        Self::new()
    }
}

/// The result of resolving a `DynamicInput` against the current mouse state.
#[derive(Debug, Clone, Copy)]
pub struct DynamicResult {
    /// The final computed point (origin + direction × distance).
    pub point: DVec2,
    /// The distance used (either override or raw).
    pub distance: f64,
    /// The angle used in degrees (0–360).
    pub angle_deg: f64,
    /// Whether polar snapping was applied.
    pub snapped: bool,
    /// Whether the distance came from a numeric override.
    pub distance_overridden: bool,
    /// Whether the angle came from a numeric override.
    pub angle_overridden: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn polar_snap_45() -> PolarSnap {
        PolarSnap::default()
    }

    #[test]
    fn test_no_overrides_raw_mouse() {
        let input = DynamicInput::new();
        let origin = DVec2::new(0.0, 0.0);
        let mouse = DVec2::new(3.0, 4.0);
        let result = input.resolve(origin, mouse, None);
        assert!((result.point.distance(origin) - 5.0).abs() < 1e-9);
        assert!(!result.snapped);
        assert!(!result.distance_overridden);
        assert!(!result.angle_overridden);
    }

    #[test]
    fn test_distance_override() {
        let mut input = DynamicInput::new();
        input.set_distance(10.0);
        let origin = DVec2::new(0.0, 0.0);
        let mouse = DVec2::new(3.0, 4.0);
        let result = input.resolve(origin, mouse, None);
        // Distance should be 10, direction follows mouse (~53.13°).
        assert!((result.distance - 10.0).abs() < 1e-9);
        assert!((result.point.x - 6.0).abs() < 1e-6);
        assert!((result.point.y - 8.0).abs() < 1e-6);
        assert!(result.distance_overridden);
    }

    #[test]
    fn test_polar_snap_45_degrees() {
        let input = DynamicInput::new();
        let snap = polar_snap_45();
        let origin = DVec2::new(0.0, 0.0);
        // ~42.9° — near 45°, should snap.
        let mouse = DVec2::new(7.0, 6.5);
        let result = input.resolve(origin, mouse, Some(&snap));
        assert!(result.snapped);
        assert!((result.angle_deg - 45.0).abs() < 1.0);
        // Distance should be the raw mouse distance (no override).
        let raw_dist = mouse.distance(origin);
        assert!((result.distance - raw_dist).abs() < 1e-9);
    }

    #[test]
    fn test_distance_override_with_polar_snap() {
        let mut input = DynamicInput::new();
        input.set_distance(50.0);
        let snap = polar_snap_45();
        let origin = DVec2::new(0.0, 0.0);
        let mouse = DVec2::new(7.0, 6.5); // near 45°
        let result = input.resolve(origin, mouse, Some(&snap));
        assert!(result.snapped);
        assert!(result.distance_overridden);
        assert!((result.distance - 50.0).abs() < 1e-9);
        // Point should be at 45° × 50mm.
        let expected = DVec2::new(
            50.0 * 45.0_f64.to_radians().cos(),
            50.0 * 45.0_f64.to_radians().sin(),
        );
        assert!((result.point.distance(expected)).abs() < 1e-9);
    }

    #[test]
    fn test_angle_override() {
        let mut input = DynamicInput::new();
        input.set_angle_deg(90.0);
        let origin = DVec2::new(0.0, 0.0);
        let mouse = DVec2::new(10.0, 3.0);
        let result = input.resolve(origin, mouse, None);
        assert!(result.angle_overridden);
        // Angle must be 90° regardless of mouse position.
        assert!((result.angle_deg - 90.0).abs() < 1.0);
        // Distance should be the raw distance (~10.44).
        let raw_dist = mouse.distance(origin);
        assert!((result.distance - raw_dist).abs() < 1e-9);
        // Point should be straight up from origin.
        assert!((result.point.x - 0.0).abs() < 1e-9);
        assert!((result.point.y - raw_dist).abs() < 1e-9);
    }

    #[test]
    fn test_distance_and_angle_override() {
        let mut input = DynamicInput::new();
        input.set_distance(30.0);
        input.set_angle_deg(30.0);
        let origin = DVec2::new(0.0, 0.0);
        let mouse = DVec2::new(100.0, 100.0);
        let result = input.resolve(origin, mouse, None);
        assert!(result.distance_overridden);
        assert!(result.angle_overridden);
        let expected = DVec2::new(
            30.0 * 30.0_f64.to_radians().cos(),
            30.0 * 30.0_f64.to_radians().sin(),
        );
        assert!((result.point.distance(expected)).abs() < 1e-9);
    }

    #[test]
    fn test_clear_resets_overrides() {
        let mut input = DynamicInput::new();
        input.set_distance(50.0);
        input.set_angle_deg(45.0);
        assert!(input.is_active());
        input.clear();
        assert!(!input.is_active());
        assert!(input.distance.is_none());
        assert!(input.angle_deg.is_none());
    }

    #[test]
    fn test_zero_origin() {
        let input = DynamicInput::new();
        let origin = DVec2::new(0.0, 0.0);
        let mouse = DVec2::new(0.0, 0.0);
        let result = input.resolve(origin, mouse, None);
        assert!((result.distance - 0.0).abs() < 1e-9);
        assert!((result.point.x - 0.0).abs() < 1e-9);
        assert!((result.point.y - 0.0).abs() < 1e-9);
    }
}
