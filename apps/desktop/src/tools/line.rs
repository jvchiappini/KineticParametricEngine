//! Line / Polyline tool — SketchUp–style.
//!
//! Behaviour:
//!  • First click anchors the polyline and plane.
//!  • Each subsequent click adds a vertex.
//!  • If the cursor snaps to the first vertex (loop closed): the polygon is
//!    tessellated into a flat `MeshDef` node (like a face in SketchUp) that
//!    the Push‑Pull tool can then extrude.
//!  • ESC or double‑click cancels the current segment without closing.
//!
//! Axis Inference:
//!  • Red / Green / Blue dotted snapping to X / Y / Z axes from the previous
//!    vertex, exactly as SketchUp's "Inference Engine".
//!  • Threshold = 5 % of camera distance.

use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use kpe_schema::geometry::{GeometryNode, GeometryNodeType, MeshDef};
use kpe_parametric::commands::AddFeatureCommand;

use crate::{app::AppState, sync::MeshNodeId, tools::core::*};

// ---------------------------------------------------------------------------
// Snap helpers
// ---------------------------------------------------------------------------

/// Snap threshold as a fraction of camera distance.
const SNAP_FRAC: f32 = 0.05;
/// Vertex snap threshold (fraction of camera distance).
const VERTEX_SNAP_FRAC: f32 = 0.03;

fn resolve_point(
    ray_o: Vec3,
    ray_d: Vec3,
    prev: Vec3,
    plane: &ConstructionPlane,
    cam_dist: f32,
    endpoints: &[Vec3], // all poly vertices so far (for vertex snap)
    snaps: &crate::app::SnappingConfig,
) -> (Vec3, SnapHint) {
    // 1 – vertex snap (endpoint / first-vertex / midpoints)
    if snaps.endpoint {
        for &v in endpoints.iter() {
            let t = (v - ray_o).dot(ray_d).max(0.0);
            let closest = ray_o + ray_d * t;
            if (closest - v).length() < cam_dist * VERTEX_SNAP_FRAC {
                return (v, SnapHint::Endpoint);
            }
        }
    }

    // 2 – midpoint snap
    if snaps.midpoint && endpoints.len() >= 2 {
        for i in 0..endpoints.len() - 1 {
            let mid = (endpoints[i] + endpoints[i + 1]) * 0.5;
            let t = (mid - ray_o).dot(ray_d).max(0.0);
            let closest = ray_o + ray_d * t;
            if (closest - mid).length() < cam_dist * VERTEX_SNAP_FRAC {
                return (mid, SnapHint::Midpoint);
            }
        }
    }

    // 3 – axis inference from previous vertex
    if snaps.axes {
        if let Some(snapped) = snap_to_axes(ray_o, ray_d, prev, cam_dist * SNAP_FRAC) {
            let axis_hint = infer_axis_hint(snapped - prev);
            return (snapped, axis_hint);
        }
    }

    // 4 – plane intersection
    let p = plane.intersect_ray(ray_o, ray_d).unwrap_or(prev);
    (p, SnapHint::None)
}

fn infer_axis_hint(delta: Vec3) -> SnapHint {
    let ax = delta.x.abs();
    let ay = delta.y.abs();
    let az = delta.z.abs();
    if ax >= ay && ax >= az { SnapHint::AxisX }
    else if ay >= ax && ay >= az { SnapHint::AxisY }
    else { SnapHint::AxisZ }
}

#[derive(Clone, Copy, PartialEq)]
pub enum SnapHint {
    None,
    Endpoint,
    Midpoint,
    AxisX,
    AxisY,
    AxisZ,
}

// ---------------------------------------------------------------------------
// Polygon tessellation — fan triangulation (convex polygons only)
// ---------------------------------------------------------------------------

fn fan_triangulate(pts: &[Vec3]) -> (Vec<[f64; 3]>, Vec<[u32; 3]>) {
    let n = pts.len();
    let mut verts: Vec<[f64; 3]> = pts.iter().map(|p| [p.x as f64, p.y as f64, p.z as f64]).collect();

    // Calculate centroid for double-face (top and bottom if coplanar)
    let centroid = pts.iter().fold(Vec3::ZERO, |a, &p| a + p) / n as f32;
    verts.push([centroid.x as f64, centroid.y as f64, centroid.z as f64]);

    let mut tris: Vec<[u32; 3]> = Vec::new();
    let c_idx = n as u32;
    // Front face
    for i in 0..n {
        let j = (i + 1) % n;
        tris.push([i as u32, j as u32, c_idx]);
    }
    // Back face (reverse winding)
    for i in 0..n {
        let j = (i + 1) % n;
        tris.push([j as u32, i as u32, c_idx]);
    }
    (verts, tris)
}

// ---------------------------------------------------------------------------
// Line tool system
// ---------------------------------------------------------------------------

pub fn line_tool_system(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform, &crate::camera::OrbitCamera)>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    meshes_query: Query<(&MeshNodeId, &GlobalTransform, &Aabb)>,
    mut state: ResMut<AppState>,
    mut tool_state: ResMut<BuildToolState>,
    editor: Res<crate::sketch_editor::SketchEditorState>,
) {
    if editor.active { return; }
    if tool_state.active_tool != BuildTool::Line { return; }

    let Some((ray_o, ray_d, cam_dist)) = cameras.get_single().ok().and_then(|(cam, ct, orbit)| {
        let cursor = windows.get_single().ok()?.cursor_position()?;
        if !in_viewport(cursor, windows.get_single().ok()?) { return None; }
        let ray = cam.viewport_to_world(ct, cursor).ok()?;
        Some((ray.origin, *ray.direction, orbit.distance))
    }) else {
        return;
    };

    // ── ESC: cancel current segment ──────────────────────────────────────
    if keys.just_pressed(KeyCode::Escape) {
        tool_state.phase = ToolPhase::Idle;
        return;
    }

    if !mouse.just_pressed(MouseButton::Left) { return; }

    let snaps = state.snaps;

    match tool_state.phase.clone() {
        ToolPhase::Idle => {
            let plane = infer_plane(ray_o, ray_d, &meshes_query, &state);
            let Some(p1) = plane.intersect_ray(ray_o, ray_d) else { return; };
            tool_state.phase = ToolPhase::Polylining {
                prev_world: p1,
                plane,
                poly_points: vec![p1],
            };
        }

        ToolPhase::Polylining { prev_world, plane, poly_points } => {
            let (hit, hint) = resolve_point(
                ray_o, ray_d, prev_world, &plane, cam_dist,
                &poly_points, &snaps,
            );
            let _ = hint;

            // ── Check if closing the loop ─────────────────────────────
            let first = poly_points[0];
            let is_closing = (hit - first).length() < cam_dist * VERTEX_SNAP_FRAC
                && poly_points.len() >= 3;

            if is_closing {
                // Tessellate polygon → MeshDef
                let (vertices, indices) = fan_triangulate(&poly_points);
                let scene = state.document.to_scene();
                let counter = kpe_parametric::next_counter(&scene.scene, "Face_");
                let node_id = format!("Face_{:03}", counter);

                let node = GeometryNode {
                    id: node_id.clone(),
                    node_type: GeometryNodeType::Mesh(MeshDef { vertices, indices }),
                    transform: None,
                    children: vec![],
                    operations: vec![],
                    color: None,
                };
                let cmd = AddFeatureCommand { parent_id: "Root".to_string(), node };
                state.execute(Box::new(cmd));
                state.document.selection = Some(node_id);
                tool_state.phase = ToolPhase::Idle;
            } else {
                // Add a new vertex and continue the polyline
                let mut new_pts = poly_points.clone();
                new_pts.push(hit);
                tool_state.phase = ToolPhase::Polylining {
                    prev_world: hit,
                    plane,
                    poly_points: new_pts,
                };
            }
        }

        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Line tool preview gizmo
// ---------------------------------------------------------------------------

pub fn line_preview_system(
    mut gizmos: Gizmos,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform, &crate::camera::OrbitCamera)>,
    tool_state: Res<BuildToolState>,
    state: Res<AppState>,
    meshes_query: Query<(&MeshNodeId, &GlobalTransform, &Aabb)>,
) {
    if tool_state.active_tool != BuildTool::Line { return; }

    let ToolPhase::Polylining { prev_world, plane, poly_points } = &tool_state.phase else { return; };

    let Some((ray_o, ray_d, cam_dist)) = cameras.get_single().ok().and_then(|(cam, ct, orbit)| {
        let cursor = windows.get_single().ok()?.cursor_position()?;
        if !in_viewport(cursor, windows.get_single().ok()?) { return None; }
        let ray = cam.viewport_to_world(ct, cursor).ok()?;
        Some((ray.origin, *ray.direction, orbit.distance))
    }) else { return; };

    let snaps = state.snaps;

    let (cursor_pt, hint) = resolve_point(ray_o, ray_d, *prev_world, plane, cam_dist, poly_points, &snaps);

    // Pick color based on axis inference
    let seg_color = match hint {
        SnapHint::AxisX  => Color::srgb(1.0, 0.18, 0.18),  // Red  = X
        SnapHint::AxisY  => Color::srgb(0.18, 1.0, 0.18),  // Green = Y
        SnapHint::AxisZ  => Color::srgb(0.18, 0.55, 1.0),  // Blue = Z
        SnapHint::Endpoint | SnapHint::Midpoint => Color::srgb(1.0, 1.0, 0.0), // Yellow snap
        SnapHint::None   => Color::srgb(0.0, 0.8, 0.9),
    };

    // Draw already-placed segments
    for i in 0..(poly_points.len().saturating_sub(1)) {
        gizmos.line(poly_points[i], poly_points[i + 1], Color::srgb(0.0, 0.8, 0.9));
    }

    // Draw current rubber-band segment
    if !poly_points.is_empty() {
        gizmos.line(*prev_world, cursor_pt, seg_color);
    }

    // Draw snap marker on cursor point
    let marker_size = cam_dist * 0.012;
    gizmos.line(cursor_pt - Vec3::X * marker_size, cursor_pt + Vec3::X * marker_size, Color::WHITE);
    gizmos.line(cursor_pt - Vec3::Y * marker_size, cursor_pt + Vec3::Y * marker_size, Color::WHITE);
    gizmos.line(cursor_pt - Vec3::Z * marker_size, cursor_pt + Vec3::Z * marker_size, Color::WHITE);

    // Highlight close-to-first-vertex snap
    let first = poly_points[0];
    if (cursor_pt - first).length() < cam_dist * VERTEX_SNAP_FRAC && poly_points.len() >= 3 {
        let r = cam_dist * 0.02;
        // Draw a "close loop" diamond indicator
        gizmos.line(first + Vec3::X * r, first + Vec3::Y * r, Color::srgb(0.0, 1.0, 0.5));
        gizmos.line(first + Vec3::Y * r, first - Vec3::X * r, Color::srgb(0.0, 1.0, 0.5));
        gizmos.line(first - Vec3::X * r, first - Vec3::Y * r, Color::srgb(0.0, 1.0, 0.5));
        gizmos.line(first - Vec3::Y * r, first + Vec3::X * r, Color::srgb(0.0, 1.0, 0.5));
    }
}
