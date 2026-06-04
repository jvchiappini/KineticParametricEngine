//! Unit system for KPE.
//!
//! **Convention:** 1 internal unit = 1 mm.
//!
//! All geometry, parameters and distances are stored in millimeters.
//! Display helpers in this module format values for the UI.

/// Format a millimeter value for display.
/// - Below 10 mm: 2 decimal places  →  "3.14 mm"
/// - 10–999 mm: 1 decimal            →  "25.0 mm"
/// - 1000+ mm: show in cm            →  "1500 mm  (150.0 cm)"
pub fn fmt_mm(value: f64) -> String {
    if value.abs() < 10.0 {
        format!("{:.2} mm", value)
    } else if value.abs() < 1000.0 {
        format!("{:.1} mm", value)
    } else {
        format!("{:.0} mm  ({:.1} cm)", value, value / 10.0)
    }
}

/// Format a distance for the VCB (compact, no unit suffix).
/// Used where context is already clear (e.g. "Push/Pull  ← 3.14").
pub fn fmt_distance(value: f64) -> String {
    if value.abs() < 10.0 {
        format!("{:.2}", value)
    } else {
        format!("{:.1}", value)
    }
}

/// Format a box as "W × H × D mm".
pub fn fmt_box(w: f64, h: f64, d: f64) -> String {
    format!("{} × {} × {}", fmt_compact(w), fmt_compact(h), fmt_compact(d))
}

/// Compact number format: removes trailing zeros.
pub fn fmt_compact(v: f64) -> String {
    if (v - v.round()).abs() < 0.005 {
        format!("{:.0}", v)
    } else if (v * 10.0 - (v * 10.0).round()).abs() < 0.05 {
        format!("{:.1}", v)
    } else {
        format!("{:.2}", v)
    }
}

/// Snap a value to the nearest `step` mm.
pub fn snap_to(value: f64, step: f64) -> f64 {
    (value / step).round() * step
}

/// Choose automatic grid step based on camera distance.
/// Returns (major_step_mm, minor_step_mm).
pub fn grid_steps(camera_distance: f32) -> (f32, f32) {
    if camera_distance < 50.0 {
        (10.0, 1.0)          // 1 cm major, 1 mm minor
    } else if camera_distance < 200.0 {
        (100.0, 10.0)        // 10 cm major, 1 cm minor
    } else if camera_distance < 1000.0 {
        (1000.0, 100.0)      // 1 m major, 10 cm minor
    } else {
        (10000.0, 1000.0)    // 10 m major, 1 m minor
    }
}
