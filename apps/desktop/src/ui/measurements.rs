//! SketchUp‑style Measurements / Value Control Box (VCB).
//!
//! Renders a translucent overlay at the bottom‑right corner of the viewport
//! showing the active tool's current value (dimensions, distance, etc.).

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::app::AppState;
use crate::build_tool::BuildToolState;
use crate::camera::OrbitCamera;
use crate::tools::core::{BuildTool, ToolPhase};
use crate::tools::push_pull::PushPullToolState;
use crate::units;

/// SketchUp‑style measurements overlay at the bottom‑right of the viewport.
pub fn measurements_ui_system(
    mut contexts: EguiContexts,
    tool_state: Res<BuildToolState>,
    push_pull: Res<PushPullToolState>,
    state: Res<AppState>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<OrbitCamera>>,
) {
    let ctx = contexts.ctx_mut();

    // ── Gather data ─────────────────────────────────────────────────
    let (line1, line2) = match &tool_state.phase {
        ToolPhase::Idle => {
            // Show selected node info, or just the tool name.
            match &state.document.selection {
                Some(sel_id) => {
                    let info = selection_info(sel_id, &state);
                    (format!("{}", tool_state.active_tool.label()), info)
                }
                None => (tool_state.active_tool.label().to_string(), String::new()),
            }
        }
        ToolPhase::Placing { p1_world, plane } => {
            let cursor = cursor_on_plane(&windows, &cameras, plane);
            let (line1, line2) = match tool_state.active_tool {
                BuildTool::Rectangle => {
                    if let Some(cursor_world) = cursor {
                        let local = plane.to_2d(cursor_world);
                        let p1_2d = plane.to_2d(*p1_world);
                        let delta = local - p1_2d;
                        let w = delta.x.abs() as f64;
                        let d = delta.y.abs() as f64;
                        ("Rectangle".to_string(), format!("W: {}  D: {}", units::fmt_mm(w), units::fmt_mm(d)))
                    } else {
                        ("Rectangle".to_string(), String::new())
                    }
                }
                BuildTool::Circle => {
                    if let Some(cursor_world) = cursor {
                        let r = (cursor_world - *p1_world).length() as f64;
                        ("Circle".to_string(), format!("R: {}", units::fmt_mm(r)))
                    } else {
                        ("Circle".to_string(), String::new())
                    }
                }
                BuildTool::Line => {
                    if let Some(cursor_world) = cursor {
                        let len = (cursor_world - *p1_world).length() as f64;
                        ("Line".to_string(), format!("{}", units::fmt_mm(len)))
                    } else {
                        ("Line".to_string(), String::new())
                    }
                }
                _ => (tool_state.active_tool.label().to_string(), String::new()),
            };
            (line1, line2)
        }
        ToolPhase::Polylining { prev_world, plane, .. } => {
            let cursor = cursor_on_plane(&windows, &cameras, plane);
            let len = cursor
                .map(|c| (c - *prev_world).length() as f64)
                .unwrap_or(0.0);
            ("Polyline".to_string(), format!("Length: {:.1}", len))
        }
        ToolPhase::Dragging { start_world, .. } => {
            if tool_state.active_tool == BuildTool::Move {
                let cursor = cursor_ray_plane(&windows, &cameras, *start_world, Vec3::Y);
                let dist = cursor
                    .map(|c| (c - *start_world).length() as f64)
                    .unwrap_or(0.0);
                ("Move".to_string(), units::fmt_mm(dist))
            } else {
                ("Move".to_string(), String::new())
            }
        }
    };

    // PushPull dragging overrides the phase data.
    let (line1, line2) = if push_pull.dragging {
        let dist = push_pull.accumulated_distance;
        if push_pull.is_parametric {
            let param_label = match push_pull.param_name.as_str() {
                "width" => "W",
                "height" => "H",
                "depth" => "D",
                "radius" => "R",
                _ => &push_pull.param_name,
            };
            let new_val = push_pull.old_param_value + dist;
            ("Push/Pull".to_string(), format!("{}: {}", param_label, units::fmt_mm(new_val)))
        } else {
            ("Push/Pull".to_string(), format!("Δ {}", units::fmt_mm(dist)))
        }
    } else {
        (line1, line2)
    };

    // ── Render ──────────────────────────────────────────────────────
    let window_width = windows.get_single().map(|w| w.resolution.width()).unwrap_or(1200.0);
    let panel_width = 220.0_f32.min(window_width * 0.18);

    egui::Area::new(egui::Id::new("measurements_box"))
        .anchor(egui::Align2::RIGHT_BOTTOM, [-12.0, -20.0])
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let frame = egui::Frame::none()
                .fill(egui::Color32::from_rgb(38, 38, 38))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 70, 70)))
                .rounding(5.0)
                .inner_margin(egui::Margin::symmetric(12.0, 8.0));
            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    if !line1.is_empty() {
                        ui.label(
                            egui::RichText::new(&line1)
                                .size(13.0)
                                .color(egui::Color32::from_gray(160))
                        );
                    }
                    if !line2.is_empty() {
                        ui.label(
                            egui::RichText::new(&line2)
                                .size(14.0)
                                .color(egui::Color32::WHITE)
                                .monospace()
                        );
                    }
                });
            });
        });
}

// ── Helpers ─────────────────────────────────────────────────────────────

/// Project the mouse cursor onto a construction plane.
fn cursor_on_plane(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform), With<OrbitCamera>>,
    plane: &crate::tools::core::ConstructionPlane,
) -> Option<Vec3> {
    let window = windows.get_single().ok()?;
    let cursor = window.cursor_position()?;
    let (cam, cam_transform) = cameras.get_single().ok()?;
    let Ok(ray) = cam.viewport_to_world(cam_transform, cursor) else {
        return None;
    };
    plane.intersect_ray(ray.origin, *ray.direction)
}

/// Project the mouse cursor onto an implicit ground plane.
fn cursor_ray_plane(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform), With<OrbitCamera>>,
    origin: Vec3,
    normal: Vec3,
) -> Option<Vec3> {
    let window = windows.get_single().ok()?;
    let cursor = window.cursor_position()?;
    let (cam, cam_transform) = cameras.get_single().ok()?;
    let Ok(ray) = cam.viewport_to_world(cam_transform, cursor) else {
        return None;
    };
    let denom = normal.dot(*ray.direction);
    if denom.abs() < 1e-6 {
        return None;
    }
    let t = (origin - ray.origin).dot(normal) / denom;
    if t < 0.0 {
        return None;
    }
    Some(ray.origin + *ray.direction * t)
}

/// Get a one‑line description for a selected node.
fn selection_info(sel_id: &str, state: &AppState) -> String {
    use kpe_geometry::evaluator::find_node;
    let scene = &state.document.recipe.scene;
    let Some(node) = find_node(scene, sel_id) else {
        return String::new();
    };
    use kpe_schema::geometry::GeometryNodeType;
    match &node.node_type {
        GeometryNodeType::Box(b) => {
            format!("Box: {}", crate::units::fmt_box(b.width, b.height, b.depth))
        }
        GeometryNodeType::Cylinder(c) => {
            if c.height > 0.01 {
                format!("Cyl: r{:.1} × h{:.1}", c.radius, c.height)
            } else {
                format!("Circle: r{:.1}", c.radius)
            }
        }
        GeometryNodeType::Sphere(s) => {
            format!("Sphere: r{:.1}", s.radius)
        }
        GeometryNodeType::Extrude(e) => {
            format!("Extrude: {:.1}", e.distance)
        }
        GeometryNodeType::Revolve(r) => {
            format!("Revolve: {:.1}°", r.angle.to_degrees())
        }
        GeometryNodeType::Compound => "Group".to_string(),
        GeometryNodeType::Assembly(_) => "Component".to_string(),
        _ => format!("{:?}", node.node_type),
    }
}
