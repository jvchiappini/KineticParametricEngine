//! Legacy `build_tool` module — now a thin wrapper around `tools::core`.
//!
//! All shared types (`BuildTool`, `ConstructionPlane`, `ToolPhase`,
//! `BuildToolState`) and helpers have moved to `tools::core` for a
//! cleaner per‑tool architecture.
//!
//! This file retains only:
//!  - Re‑exports so existing code (`crate::build_tool::*`) continues to work.
//!  - The keyboard‑shortcut system.
//!  - The gizmo‑rendering system (preview rubber‑band lines).
//!  - The tool‑palette egui window (to be replaced by vertical toolbar).

pub use crate::tools::core::{
    BuildTool, BuildToolState, ConstructionPlane, ToolPhase, dmat4_to_mat4, get_ray,
    infer_plane, in_viewport, is_push_pull_active, is_select_active, pick_face, pick_node_aabb,
    ray_triangle_intersection,
};

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Keyboard shortcut system
// ---------------------------------------------------------------------------

pub fn tool_shortcut_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut tool_state: ResMut<BuildToolState>,
) {
    let tools = [
        BuildTool::Select,
        BuildTool::Rectangle,
        BuildTool::Circle,
        BuildTool::Line,
        BuildTool::PushPull,
        BuildTool::Move,
        BuildTool::Eraser,
    ];
    for tool in &tools {
        if keys.just_pressed(tool.keyboard_key()) {
            tool_state.active_tool = *tool;
            tool_state.phase = ToolPhase::Idle;
            return;
        }
    }
    if keys.just_pressed(KeyCode::Escape) {
        if tool_state.phase != ToolPhase::Idle {
            tool_state.phase = ToolPhase::Idle;
        } else {
            tool_state.active_tool = BuildTool::Select;
        }
    }
}

// ---------------------------------------------------------------------------
// Gizmo rendering (rubber‑band previews)
// ---------------------------------------------------------------------------

pub fn tool_render_system(
    mut gizmos: Gizmos,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform, &crate::camera::OrbitCamera)>,
    tool_state: Res<BuildToolState>,
) {
    fn get_cursor_ray_dist(
        windows: &Query<&Window>,
        cameras: &Query<(&Camera, &GlobalTransform, &crate::camera::OrbitCamera)>,
    ) -> Option<(Vec3, Vec3, f32)> {
        let window = windows.get_single().ok()?;
        let cursor = window.cursor_position()?;
        let (cam, ct, orbit) = cameras.get_single().ok()?;
        let Ok(ray) = cam.viewport_to_world(ct, cursor) else {
            return None;
        };
        Some((ray.origin, *ray.direction, orbit.distance))
    }

    let Some((ray_o, ray_d, cam_dist)) = get_cursor_ray_dist(&windows, &cameras) else {
        return;
    };

    match &tool_state.phase {
        ToolPhase::Placing { p1_world, plane } => {
            let cursor_plane = plane.intersect_ray(ray_o, ray_d).unwrap_or(*p1_world);
            let cproj = plane.project(cursor_plane);
            match tool_state.active_tool {
                BuildTool::Rectangle => draw_rect_preview(&mut gizmos, *p1_world, cproj, plane),
                BuildTool::Circle => draw_circle_preview(&mut gizmos, *p1_world, cproj, plane),
                BuildTool::Line => {
                    let mut p2 = cproj;
                    // Snapping logic identical to the line tool
                    if let Some(snapped) = crate::tools::core::snap_to_axes(ray_o, ray_d, *p1_world, cam_dist * 0.05) {
                        p2 = snapped;
                    }
                    gizmos.line(*p1_world, p2, Color::srgb(0.0, 0.8, 0.9));
                }
                _ => {}
            }
        }
        ToolPhase::Polylining { prev_world, plane, .. } => {
            if let Some(cursor_plane) = plane.intersect_ray(ray_o, ray_d) {
                gizmos.line(*prev_world, cursor_plane, Color::srgb(0.0, 0.8, 0.9));
            }
        }
        _ => {}
    }
}

fn draw_rect_preview(gizmos: &mut Gizmos, p1: Vec3, p2: Vec3, plane: &ConstructionPlane) {
    let local1 = plane.to_2d(p1);
    let local2 = plane.to_2d(p2);
    let min = local1.min(local2);
    let max = local1.max(local2);
    
    let c0 = plane.origin + plane.u_axis * min.x + plane.v_axis * min.y;
    let c1 = plane.origin + plane.u_axis * max.x + plane.v_axis * min.y;
    let c2 = plane.origin + plane.u_axis * max.x + plane.v_axis * max.y;
    let c3 = plane.origin + plane.u_axis * min.x + plane.v_axis * max.y;
    
    let corners = [c0, c1, c2, c3];
    let color = Color::srgb(0.0, 0.8, 0.9);
    for i in 0..4 {
        gizmos.line(corners[i], corners[(i + 1) % 4], color);
    }
    gizmos.line(p1, p2, Color::srgb(0.3, 0.3, 0.3));
}

fn draw_circle_preview(gizmos: &mut Gizmos, center: Vec3, cursor: Vec3, plane: &ConstructionPlane) {
    let radius = (cursor - center).length();
    if radius < 0.001 {
        return;
    }
    let color = Color::srgb(0.0, 0.8, 0.9);
    let segments = 64;
    
    let u = plane.u_axis;
    let v = plane.v_axis;

    for i in 0..segments {
        let a1 = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let a2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;
        gizmos.line(
            center + (u * a1.cos() + v * a1.sin()) * radius,
            center + (u * a2.cos() + v * a2.sin()) * radius,
            color,
        );
    }
    gizmos.line(center, cursor, Color::srgb(0.3, 0.3, 0.3));
}

// ---------------------------------------------------------------------------
// Vertical Toolbar (SketchUp 2023+ style)
// ---------------------------------------------------------------------------

/// SketchUp‑style vertical toolbar docked to the left edge of the viewport.
///
/// Renders large icon‑only tool buttons in a single narrow column.
pub fn vertical_toolbar_ui_system(
    mut contexts: bevy_egui::EguiContexts,
    mut tool_state: ResMut<BuildToolState>,
) {
    use bevy_egui::egui;

    let ctx = contexts.ctx_mut();
    let toolbar_width = 52.0;

    egui::Area::new(egui::Id::new("vertical_toolbar"))
        .anchor(egui::Align2::LEFT_TOP, [8.0, 42.0])
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.set_max_width(toolbar_width);
            ui.set_min_width(toolbar_width);
            let frame = egui::Frame::none()
                .fill(egui::Color32::from_rgb(38, 38, 38))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 70, 70)))
                .rounding(5.0)
                .inner_margin(egui::Margin::symmetric(4.0, 6.0));
            frame.show(ui, |ui| {

                let tools = [
                    (BuildTool::Select,   "Sel",  "Select [Space]",    'S'),
                    (BuildTool::Rectangle,"Rect", "Rectangle [R]",     'R'),
                    (BuildTool::Circle,   "Circ", "Circle [C]",        'C'),
                    (BuildTool::Line,     "Line", "Line [L]",          'L'),
                    (BuildTool::PushPull, "P/P",  "Push/Pull [P]",     'P'),
                    (BuildTool::Move,     "Move", "Move [M]",          'M'),
                    (BuildTool::Eraser,   "Erase","Eraser [E]",        'E'),
                ];

                for (tool, short, hover, _key) in &tools {
                    let is_active = tool_state.active_tool == *tool;
                    let btn = egui::Button::new(
                        egui::RichText::new(*short)
                            .size(13.0)
                            .color(if is_active {
                                egui::Color32::WHITE
                            } else {
                                egui::Color32::from_gray(210)
                            })
                    )
                        .fill(if is_active {
                            egui::Color32::from_rgb(50, 120, 210)
                        } else {
                            egui::Color32::from_gray(30)
                        })
                        .stroke(if is_active {
                            egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 180, 255))
                        } else {
                            egui::Stroke::new(1.0, egui::Color32::from_gray(50))
                        })
                        .min_size(egui::vec2(toolbar_width - 12.0, 30.0));
                    if ui.add(btn).on_hover_text(*hover).clicked() {
                        tool_state.active_tool = *tool;
                        tool_state.phase = ToolPhase::Idle;
                    }
                }
            });
        });

    // Phase info tooltip — below the toolbar
    if tool_state.phase != ToolPhase::Idle {
        let help = match &tool_state.phase {
            ToolPhase::Placing { .. } => "Click to place second point • ESC to cancel",
            ToolPhase::Polylining { .. } => "Click to continue polyline • ESC to end",
            ToolPhase::Dragging { .. } => "Drag to complete operation",
            _ => "",
        };
        let ctx = contexts.ctx_mut();
        egui::Area::new(egui::Id::new("tool_help"))
            .anchor(egui::Align2::LEFT_BOTTOM, [12.0, -20.0])
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let frame = egui::Frame::none()
                    .fill(egui::Color32::from_rgb(38, 38, 38))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 70, 70)))
                    .rounding(5.0)
                    .inner_margin(egui::Margin::symmetric(10.0, 6.0));
                frame.show(ui, |ui| {
                    ui.label(egui::RichText::new(help).size(13.0).color(egui::Color32::from_gray(200)));
                });
            });
    }
}
