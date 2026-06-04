// STUB
// Face-level ray-triangle picking and push-pull extrusion for the 3D viewport.
// Follows SketchUp-style interaction:
//   Shift+Click → select face via ray-triangle intersection
//   Click+drag on selected face → extrude (push-pull)
//   Shift+click empty space → deselect
//
// This module operates directly on `AppState.document.evaluated.meshes` so that
// extrude results are picked up by the mesh sync system when `mark_dirty()` is
// called.

use bevy::prelude::*;
use kpe_geometry::push_pull::extrude_face;
use kpe_schema::geometry::TriangleMesh;

use crate::{app::AppState, build_tool, camera, sync::MeshNodeId};

// ── Resources ───────────────────────────────────────────────────────

/// Tracks face selection and push-pull drag state for the 3D viewport.
#[derive(Resource)]
pub struct PushPullState {
    /// Currently selected face: `(node_id, triangle_index)`.
    pub selected_face: Option<(String, usize)>,

    /// Whether a push-pull drag is currently in progress.
    pub dragging: bool,

    /// Copy of the mesh at the moment the drag started, so we can re-extrude
    /// from a clean base on each frame.
    pub original_mesh: Option<TriangleMesh>,

    /// The signed extrusion distance accumulated so far in the current drag.
    pub accumulated_distance: f64,

    /// World-space hit point on the selected face at drag start.
    pub drag_origin_world: Vec3,

    /// World-space face normal at drag start.
    pub drag_normal: Vec3,

    /// Ray parameter `t` at the initial click (distance from camera to face
    /// plane along the click ray). Used to compute extrusion distance as
    /// `current_t - drag_start_t` during drag.
    pub drag_start_t: f32,
}

impl Default for PushPullState {
    fn default() -> Self {
        Self {
            selected_face: None,
            dragging: false,
            original_mesh: None,
            accumulated_distance: 0.0,
            drag_origin_world: Vec3::ZERO,
            drag_normal: Vec3::Y,
            drag_start_t: 0.0,
        }
    }
}

// ── Ray-Triangle Intersection (Möller–Trumbore) ────────────────────

/// Returns the distance `t` from `ray_origin` along `ray_dir` to the
/// intersection point with triangle `(v0, v1, v2)`, or `None` if no
/// intersection (or if the intersection is behind the ray origin).
///
/// `ray_dir` *must* be unit-length for the returned `t` to be meaningful
/// as a world-space distance.
fn ray_triangle_intersection(
    ray_origin: Vec3,
    ray_dir: Vec3,
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
) -> Option<f32> {
    let edge1 = v1 - v0;
    let edge2 = v2 - v0;
    let h = ray_dir.cross(edge2);
    let a = edge1.dot(h);

    // Ray is parallel to triangle
    if a.abs() < 1e-10 {
        return None;
    }

    let f = 1.0 / a;
    let s = ray_origin - v0;
    let u = f * s.dot(h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let q = s.cross(edge1);
    let v = f * ray_dir.dot(q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = f * edge2.dot(q);
    if t > 1e-5 { Some(t) } else { None }
}

// ── Helper: DMat4 → Bevy Mat4 ──────────────────────────────────────

fn dmat4_to_mat4(m: glam::DMat4) -> Mat4 {
    Mat4::from_cols(
        Vec4::new(m.x_axis.x as f32, m.x_axis.y as f32, m.x_axis.z as f32, m.x_axis.w as f32),
        Vec4::new(m.y_axis.x as f32, m.y_axis.y as f32, m.y_axis.z as f32, m.y_axis.w as f32),
        Vec4::new(m.z_axis.x as f32, m.z_axis.y as f32, m.z_axis.z as f32, m.z_axis.w as f32),
        Vec4::new(m.w_axis.x as f32, m.w_axis.y as f32, m.w_axis.z as f32, m.w_axis.w as f32),
    )
}

// ── Face-Picking System ─────────────────────────────────────────────

/// Runs every frame.
///
/// When Shift+Left-click is pressed on a mesh entity, performs a full
/// ray-triangle intersection test against that mesh's `TriangleMesh` and
/// records the closest face in `PushPullState`.
///
/// If no face is hit (click on empty space) while shift is held, the
/// selection is cleared.
pub fn face_pick_system(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<camera::OrbitCamera>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    meshes_query: Query<(&MeshNodeId, &GlobalTransform)>,
    state: Res<AppState>,
    mut push_pull: ResMut<PushPullState>,
    editor: Res<crate::sketch_editor::SketchEditorState>,
    tool_state: Res<build_tool::BuildToolState>,
) {
    // Only respond when sketch editor is inactive
    if editor.active {
        return;
    }

    // Only respond to left-click
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    // Gate: only activate when PushPull tool is active, or when shift+click
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if !build_tool::is_push_pull_active(&tool_state) && !shift {
        // Not in push-pull mode and no shift — deselect face and return
        push_pull.selected_face = None;
        return;
    }

    let window = match windows.get_single() {
        Ok(w) => w,
        _ => return,
    };
    let cursor = match window.cursor_position() {
        Some(c) => c,
        None => return,
    };
    let (cam, cam_transform) = match cameras.get_single() {
        Ok(c) => c,
        _ => return,
    };
    let Ok(ray) = cam.viewport_to_world(cam_transform, cursor) else {
        return;
    };
    let ray_origin = ray.origin;
    let ray_dir = ray.direction.as_vec3();

    // Ignore clicks in the UI margins
    let viewport_size = &window.resolution;
    if cursor.x < 220.0 || cursor.x > viewport_size.width() - 280.0 {
        return;
    }

    // When PushPull tool is active, a plain click selects a face
    // If already dragging, ignore (drag system handles it)
    if push_pull.dragging {
        return;
    }

    // ── Face selection via ray-triangle intersection ────────────────

    // Abort any in-progress drag
    push_pull.dragging = false;
    push_pull.original_mesh = None;
    push_pull.accumulated_distance = 0.0;

    // Compute world matrices for the whole scene so we can transform
    // mesh vertices from local to world space.
    let matrices =
        kpe_geometry::evaluator::compute_world_matrices(
            &state.document.recipe.scene,
            glam::DMat4::IDENTITY,
        );

    let mut best: Option<(f32, String, usize, Vec3, Vec3)> = None;

    for (mesh_node_id, _transform) in &meshes_query {
        let node_id_str = &mesh_node_id.0;

        // Skip nodes that have no evaluated mesh
        let tri_mesh = match state.document.evaluated.meshes.get(node_id_str) {
            Some(m) => m,
            None => continue,
        };

        if tri_mesh.triangles.is_empty() || tri_mesh.vertices.is_empty() {
            continue;
        }

        // Build the world matrix for this node
        let world = matrices
            .get(node_id_str)
            .map(|m| dmat4_to_mat4(*m))
            .unwrap_or(Mat4::IDENTITY);

        let inv = world.inverse();
        let local_origin = inv.transform_point3(ray_origin);
        let local_dir = inv.transform_vector3(ray_dir).normalize();

        let verts = &tri_mesh.vertices;

        for (face_idx, tri) in tri_mesh.triangles.iter().enumerate() {
            let v0 = Vec3::new(
                verts[tri[0] as usize][0] as f32,
                verts[tri[0] as usize][1] as f32,
                verts[tri[0] as usize][2] as f32,
            );
            let v1 = Vec3::new(
                verts[tri[1] as usize][0] as f32,
                verts[tri[1] as usize][1] as f32,
                verts[tri[1] as usize][2] as f32,
            );
            let v2 = Vec3::new(
                verts[tri[2] as usize][0] as f32,
                verts[tri[2] as usize][1] as f32,
                verts[tri[2] as usize][2] as f32,
            );

            if let Some(t) = ray_triangle_intersection(local_origin, local_dir, v0, v1, v2) {
                let is_better = best.as_ref().map_or(true, |(d, ..)| t < *d);
                if is_better {
                    // Compute world-space hit point and face normal
                    let local_hit = local_origin + local_dir * t;
                    let world_hit = world.transform_point3(local_hit);
                    let local_normal = (v1 - v0).cross(v2 - v0).normalize();
                    let world_normal =
                        world.transform_vector3(local_normal).normalize();

                    best = Some((
                        t,
                        node_id_str.clone(),
                        face_idx,
                        world_hit,
                        world_normal,
                    ));
                }
            }
        }
    }

    if let Some((_, node_id, face_idx, hit_point, normal)) = best {
        push_pull.selected_face = Some((node_id.clone(), face_idx));
        push_pull.drag_origin_world = hit_point;
        push_pull.drag_normal = normal;
        // Store the original mesh and start dragging immediately so that
        // push_pull_drag_system can compute extrusion deltas each frame.
        if let Some(mesh) = state.document.evaluated.meshes.get(&node_id) {
            push_pull.original_mesh = Some(mesh.clone());
        }
        // Compute and store the initial ray-plane parameter t for the click
        // ray. This is the distance from camera to the face plane along the
        // click direction. During drag, extrusion = current_t - drag_start_t.
        let click_denom = normal.dot(ray_dir);
        if click_denom.abs() > 1e-6 {
            push_pull.drag_start_t = (hit_point - ray_origin).dot(normal) / click_denom;
        } else {
            push_pull.drag_start_t = (hit_point - ray_origin).length();
        }
        push_pull.dragging = true;
        push_pull.accumulated_distance = 0.0;
    } else {
        // Shift+clicked empty space → deselect
        push_pull.selected_face = None;
    }
}

// ── Push-Pull Drag System ──────────────────────────────────────────

/// Runs every frame.
///
/// When `PushPullState.dragging` is `true`, computes the extrusion distance
/// by projecting the mouse ray onto the face normal direction and calls
/// `extrude_face()` every frame.
pub fn push_pull_drag_system(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<camera::OrbitCamera>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<AppState>,
    mut push_pull: ResMut<PushPullState>,
) {
    if !push_pull.dragging {
        return;
    }

    let window = match windows.get_single() {
        Ok(w) => w,
        _ => return,
    };
    let cursor = match window.cursor_position() {
        Some(c) => c,
        None => return,
    };
    let (cam, cam_transform) = match cameras.get_single() {
        Ok(c) => c,
        _ => return,
    };
    let Ok(ray) = cam.viewport_to_world(cam_transform, cursor) else {
        return;
    };
    let ray_origin = ray.origin;
    let ray_dir = ray.direction.as_vec3();

    let sel = match &push_pull.selected_face {
        Some(s) => s.clone(),
        None => {
            push_pull.dragging = false;
            return;
        }
    };
    let (node_id, face_idx) = sel;

    let plane_normal = push_pull.drag_normal;
    let plane_origin = push_pull.drag_origin_world;

    // ── Release check ──────────────────────────────────────────────
    if mouse.just_released(MouseButton::Left) {
        // Finalize the extrusion
        if let Some(orig) = &push_pull.original_mesh {
            let result = extrude_face(orig, face_idx, push_pull.accumulated_distance);
            state.document.evaluated.meshes.insert(node_id.clone(), result);
            state.mark_dirty();
        }
        push_pull.dragging = false;
        push_pull.original_mesh = None;
        push_pull.accumulated_distance = 0.0;
        return;
    }

    // Compute extrusion distance as the difference in the ray-plane parameter
    // t between the current frame and the initial click:
    //   extrusion = current_plane_t - drag_start_t
    // where plane_t = (plane_origin - ray_origin)·normal / ray_dir·normal
    let denom = plane_normal.dot(ray_dir);
    if denom.abs() < 1e-6 {
        return; // ray parallel to face plane
    }
    let current_t = (plane_origin - ray_origin).dot(plane_normal) / denom;
    if current_t < 0.0 {
        return; // intersection behind camera
    }
    let extrusion_t: f64 = (current_t - push_pull.drag_start_t) as f64;

    // Snap to 1mm increments (configurable later)
    let snap_interval = 1.0;
    let snapped = (extrusion_t / snap_interval).round() * snap_interval;
    // Avoid micro-updates that would cause mesh thrash
    if (snapped - push_pull.accumulated_distance).abs() < 0.001 {
        return;
    }

    // Apply the extrusion: always re-extrude from the original mesh with
    // the new cumulative distance so we accumulate cleanly instead of
    // compounding errors.
    if let Some(orig) = &push_pull.original_mesh {
        let result = extrude_face(orig, face_idx, snapped);
        state.document.evaluated.meshes.insert(node_id.clone(), result);
        state.mark_dirty();
    }

    push_pull.accumulated_distance = snapped;
}

// ── Selected Face Highlight ────────────────────────────────────────

/// Draws a turquoise wireframe overlay on the currently selected face.
pub fn draw_selected_face_system(
    mut gizmos: Gizmos,
    state: Res<AppState>,
    push_pull: Res<PushPullState>,
    editor: Res<crate::sketch_editor::SketchEditorState>,
) {
    if editor.active {
        return;
    }

    let Some((ref node_id, face_idx)) = push_pull.selected_face else {
        return;
    };

    // Don't draw during drag (the mesh updates visually anyway via sync)
    if push_pull.dragging {
        return;
    }

    let tri_mesh = match state.document.evaluated.meshes.get(node_id) {
        Some(m) => m,
        None => return,
    };

    if face_idx >= tri_mesh.triangles.len() {
        return;
    }

    let matrices = kpe_geometry::evaluator::compute_world_matrices(
        &state.document.recipe.scene,
        glam::DMat4::IDENTITY,
    );
    let world = matrices
        .get(node_id)
        .map(|m| dmat4_to_mat4(*m))
        .unwrap_or(Mat4::IDENTITY);

    let tri = tri_mesh.triangles[face_idx];
    let verts = &tri_mesh.vertices;
    let v0 = world.transform_point3(Vec3::new(
        verts[tri[0] as usize][0] as f32,
        verts[tri[0] as usize][1] as f32,
        verts[tri[0] as usize][2] as f32,
    ));
    let v1 = world.transform_point3(Vec3::new(
        verts[tri[1] as usize][0] as f32,
        verts[tri[1] as usize][1] as f32,
        verts[tri[1] as usize][2] as f32,
    ));
    let v2 = world.transform_point3(Vec3::new(
        verts[tri[2] as usize][0] as f32,
        verts[tri[2] as usize][1] as f32,
        verts[tri[2] as usize][2] as f32,
    ));

    let highlight = Color::srgb(0.0, 0.95, 0.9);

    // Triangle outline (bright turquoise)
    gizmos.line(v0, v1, highlight);
    gizmos.line(v1, v2, highlight);
    gizmos.line(v2, v0, highlight);

    // Slightly larger glow around edges by drawing offset lines
    let glow = Color::srgb(0.0, 0.6, 0.6);
    let center = (v0 + v1 + v2) / 3.0;
    let offset = (center - v0).normalize_or_zero() * 0.015;
    gizmos.line(v0 + offset, v1 + offset, glow);
    gizmos.line(v1 + offset, v2 + offset, glow);
    gizmos.line(v2 + offset, v0 + offset, glow);
}
