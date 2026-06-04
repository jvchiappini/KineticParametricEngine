//! Face‑based push‑pull extrusion tool.
//!
//! Unlike the legacy `push_pull_system.rs` which extrudes individual
//! triangles, this tool:
//!
//! 1. **Detects real faces** — groups coplanar, connected triangles into
//!    `Face` structs via `kpe_geometry::face::detect_faces`.
//! 2. **Extrudes along boundary loops** — side quads are generated from
//!    the boundary edges, producing clean manifold geometry.
//! 3. **Supports multiple clicks** — each face click creates a new
//!    extrusion history step.
//!
//! Flow:
//!   Idle       → click on face → highlight face
//!   Highlight  → click again (or drag) → extrude
//!   Drag       → live extrusion preview
//!   Release    → finalise, store result

use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use kpe_geometry::evaluator::find_node;
use kpe_geometry::face::{detect_faces, extrude_face, Face};
use kpe_parametric::set_node_parameter;
use kpe_schema::geometry::{GeometryNodeType, TriangleMesh};

use crate::tools::core::{self, dmat4_to_mat4, ray_triangle_intersection, BuildTool};
use crate::{app::AppState, camera, sync::MeshNodeId};

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// Tracks face selection and push‑pull drag state for the 3D viewport.
///
/// This resource is managed by the systems in this module and should be
/// inserted as a Bevy `Resource` at startup.
#[derive(Resource)]
pub struct PushPullToolState {
    // ── Selection ──
    /// The node whose face is currently selected, if any.
    pub selected_node: Option<String>,
    /// Index of the selected face in the face list returned by `detect_faces`.
    pub selected_face_idx: usize,
    /// Cached `Face` geometry for the selected face.
    pub cached_face: Option<Face>,

    // ── Parametric push-pull ──
    /// If true, this push‑pull operation adjusts a parametric primitive's
    /// dimension (Box.height, Cylinder.radius, etc.) instead of extruding mesh.
    pub is_parametric: bool,
    /// Name of the parameter being adjusted ("width", "height", "depth",
    /// "radius").
    pub param_name: String,
    /// Value of the parameter at the moment the drag started (for undo).
    pub old_param_value: f64,
    /// Translation at the moment the drag started.
    pub old_translation: Option<[f64; 3]>,

    // ── Drag ──
    /// Whether a drag operation is in progress.
    pub dragging: bool,
    /// Copy of the mesh at the moment the drag started.
    pub original_mesh: Option<TriangleMesh>,
    /// Ray‑plane parameter `t` at the initial click.
    pub drag_start_t: f32,
    /// World‑space normal at the drag start.
    pub drag_normal: Vec3,
    /// World‑space hit point at drag start.
    pub drag_origin_world: Vec3,
    /// Cumulative signed extrusion distance.
    pub accumulated_distance: f64,
}

impl Default for PushPullToolState {
    fn default() -> Self {
        Self {
            selected_node: None,
            selected_face_idx: 0,
            cached_face: None,
            is_parametric: false,
            param_name: String::new(),
            old_param_value: 0.0,
            old_translation: None,
            dragging: false,
            original_mesh: None,
            drag_start_t: 0.0,
            drag_normal: Vec3::Y,
            drag_origin_world: Vec3::ZERO,
            accumulated_distance: 0.0,
        }
    }
}

/// Determine the parameter name for a parametric primitive based on face normal.
///
/// For Box: dominant axis of the normal → width (X), height (Y), depth (Z).
/// For Cylinder: normal ≈ ±Y → height, else → radius.
fn parametric_param_name(node_type: &GeometryNodeType, normal_local: glam::DVec3) -> Option<&'static str> {
    match node_type {
        GeometryNodeType::Box(_) => {
            let ax = normal_local.abs();
            if ax.x >= ax.y && ax.x >= ax.z {
                Some("width")
            } else if ax.y >= ax.x && ax.y >= ax.z {
                Some("height")
            } else {
                Some("depth")
            }
        }
        GeometryNodeType::Cylinder(_) => {
            if normal_local.y.abs() > 0.9 {
                Some("height")
            } else {
                Some("radius")
            }
        }
        _ => None,
    }
}

/// Read the current value of a named parameter from a GeometryNode.
fn read_param_value(node_type: &GeometryNodeType, name: &str) -> Option<f64> {
    match (node_type, name) {
        (GeometryNodeType::Box(b), "width") => Some(b.width),
        (GeometryNodeType::Box(b), "height") => Some(b.height),
        (GeometryNodeType::Box(b), "depth") => Some(b.depth),
        (GeometryNodeType::Cylinder(c), "radius") => Some(c.radius),
        (GeometryNodeType::Cylinder(c), "height") => Some(c.height),
        _ => None,
    }
}

/// Find which face index contains the given triangle index.
fn find_face_for_triangle(faces: &[Face], tri_idx: usize) -> Option<usize> {
    faces.iter().position(|f| f.triangle_indices.contains(&tri_idx))
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Face‑pick system: click on a mesh triangle → detect faces → select the face.
///
/// Activated when the legacy `BuildTool::PushPull` is active (`[P]` key).
pub fn face_pick_system(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<camera::OrbitCamera>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    meshes_query: Query<(&MeshNodeId, &GlobalTransform, &Aabb)>,
    tool_state: Res<crate::build_tool::BuildToolState>,
    mut push_pull: ResMut<PushPullToolState>,
    state: Res<AppState>,
    editor: Res<crate::sketch_editor::SketchEditorState>,
) {
    if editor.active || push_pull.dragging {
        return;
    }

    // Gate on PushPull tool or shift+click.
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if tool_state.active_tool != BuildTool::PushPull && !shift {
        push_pull.selected_node = None;
        push_pull.cached_face = None;
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

    // Ignore clicks in UI margins
    let viewport_size = &window.resolution;
    if cursor.x < 60.0 || cursor.x > viewport_size.width() - 280.0 {
        push_pull.selected_node = None;
        push_pull.cached_face = None;
        return;
    }

    // ── Ray‑pick the closest triangle across all meshes ────────────
    let matrices = kpe_geometry::evaluator::compute_world_matrices(
        &state.document.recipe.scene,
        glam::DMat4::IDENTITY,
    );

    // (t, node_id, tri_idx, world_hit, world_normal)
    let mut best: Option<(f32, String, usize, Vec3, Vec3)> = None;

    for (mesh_node_id, _transform, _aabb) in &meshes_query {
        let node_id = &mesh_node_id.0;
        let tri_mesh = match state.document.evaluated.meshes.get(node_id) {
            Some(m) => m,
            None => continue,
        };
        if tri_mesh.triangles.is_empty() || tri_mesh.vertices.is_empty() {
            continue;
        }

        let verts = &tri_mesh.vertices;
        for (tri_idx, tri) in tri_mesh.triangles.iter().enumerate() {
            let v0 = Vec3::new(verts[tri[0] as usize][0] as f32, verts[tri[0] as usize][1] as f32, verts[tri[0] as usize][2] as f32);
            let v1 = Vec3::new(verts[tri[1] as usize][0] as f32, verts[tri[1] as usize][1] as f32, verts[tri[1] as usize][2] as f32);
            let v2 = Vec3::new(verts[tri[2] as usize][0] as f32, verts[tri[2] as usize][1] as f32, verts[tri[2] as usize][2] as f32);

            if let Some(t) = ray_triangle_intersection(ray_origin, ray_dir, v0, v1, v2) {
                let better = best.as_ref().map_or(true, |(d, ..)| t < *d);
                if better {
                    let world_hit = ray_origin + ray_dir * t;
                    let world_normal = (v1 - v0).cross(v2 - v0).normalize();
                    best = Some((t, node_id.clone(), tri_idx, world_hit, world_normal));
                }
            }
        }
    }

    let (_, node_id, hit_tri_idx, hit_point, normal) = match best {
        Some(v) => v,
        None => {
            // Clicked empty space → deselect.
            push_pull.selected_node = None;
            push_pull.cached_face = None;
            return;
        }
    };

    // ── Detect faces for this mesh ─────────────────────────────────
    let tri_mesh = match state.document.evaluated.meshes.get(&node_id) {
        Some(m) => m.clone(),
        None => return,
    };

    let faces = detect_faces(&tri_mesh);
    let face_idx = match find_face_for_triangle(&faces, hit_tri_idx) {
        Some(i) => i,
        None => {
            // The triangle is not part of any face (shouldn't happen
            // with manifold meshes, but guard anyway).
            push_pull.selected_node = Some(node_id.clone());
            push_pull.selected_face_idx = 0;
            push_pull.cached_face = None;
            return;
        }
    };

    let face = faces[face_idx].clone();

    // Always update hover state
    push_pull.selected_node = Some(node_id.clone());
    push_pull.selected_face_idx = face_idx;
    push_pull.cached_face = Some(face.clone());

    if mouse.just_pressed(MouseButton::Left) {
        // ── Check if target is a parametric primitive (Box / Cylinder) ──
        let (is_parametric, param_name, old_param_value, old_translation) = {
            let scene = &state.document.recipe.scene;
            if let Some(node) = find_node(scene, &node_id) {
                let tx = kpe_parametric::commands::get_node_translation(scene, &node_id);
                // face.normal is in WORLD space. We need local space to detect parametric axis.
                let world_mat = matrices.get(&node_id).map(|m| dmat4_to_mat4(*m)).unwrap_or(Mat4::IDENTITY);
                let inv = world_mat.inverse();
                let face_normal_world = Vec3::new(face.normal.x as f32, face.normal.y as f32, face.normal.z as f32);
                let face_normal_local = inv.transform_vector3(face_normal_world).normalize();

                if let Some(pname) = parametric_param_name(&node.node_type, glam::DVec3::new(face_normal_local.x as f64, face_normal_local.y as f64, face_normal_local.z as f64)) {
                    if let Some(pval) = read_param_value(&node.node_type, pname) {
                        (true, pname.to_string(), pval, tx)
                    } else {
                        (false, String::new(), 0.0, None)
                    }
                } else {
                    (false, String::new(), 0.0, None)
                }
            } else {
                (false, String::new(), 0.0, None)
            }
        };
        push_pull.is_parametric = is_parametric;
        push_pull.param_name = param_name;
        push_pull.old_param_value = old_param_value;
        push_pull.old_translation = old_translation;

        push_pull.drag_origin_world = hit_point;
        push_pull.drag_normal = normal;

        // Store the original mesh for clean re‑extrusion.
        push_pull.original_mesh = Some(tri_mesh);

        // Compute initial ray‑plane t.
        let denom = normal.dot(ray_dir);
        push_pull.drag_start_t = if denom.abs() > 1e-6 {
            (hit_point - ray_origin).dot(normal) / denom
        } else {
            (hit_point - ray_origin).length()
        };

        // Begin dragging immediately.
        push_pull.dragging = true;
        push_pull.accumulated_distance = 0.0;
    }
}

/// Drag system: live extrusion preview while the mouse is held.
pub fn push_pull_drag_system(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<camera::OrbitCamera>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<AppState>,
    mut push_pull: ResMut<PushPullToolState>,
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

    let node_id = match &push_pull.selected_node {
        Some(n) => n.clone(),
        None => {
            push_pull.dragging = false;
            return;
        }
    };
    let face = match &push_pull.cached_face {
        Some(f) => f.clone(),
        None => {
            push_pull.dragging = false;
            return;
        }
    };

    let plane_normal = push_pull.drag_normal;
    let plane_origin = push_pull.drag_origin_world;

    // ── Release: finalise ──────────────────────────────────────────
    if mouse.just_released(MouseButton::Left) {
        if push_pull.is_parametric {
            // Push a proper CompoundCommand to the undo stack.
            let current_value = {
                let scene = &state.document.recipe.scene;
                find_node(scene, &node_id)
                    .and_then(|n| read_param_value(&n.node_type, &push_pull.param_name))
                    .unwrap_or(push_pull.old_param_value)
            };
            let current_translation = {
                let scene = &state.document.recipe.scene;
                kpe_parametric::commands::get_node_translation(scene, &node_id).unwrap_or([0.0; 3])
            };

            let cmd_param = Box::new(kpe_parametric::commands::SetParameterCommand {
                node_id: node_id.clone(),
                param_name: push_pull.param_name.clone(),
                old_value: push_pull.old_param_value,
                new_value: current_value,
            });
            let cmd_trans = Box::new(kpe_parametric::commands::MoveNodeTransformCommand {
                node_id: node_id.clone(),
                old_translation: push_pull.old_translation,
                new_translation: current_translation,
            });
            let compound = Box::new(kpe_parametric::commands::CompoundCommand {
                commands: vec![cmd_param, cmd_trans],
                label: "Parametric Push/Pull".to_string(),
            });
            state.execute(compound);
        } else {
            // Standard mesh extrusion.
            if let Some(orig) = &push_pull.original_mesh {
                let result = extrude_face(orig, &face, push_pull.accumulated_distance);
                state.document.evaluated.meshes.insert(node_id, result);
                state.mark_dirty();
            }
        }
        push_pull.dragging = false;
        push_pull.original_mesh = None;
        push_pull.accumulated_distance = 0.0;
        push_pull.is_parametric = false;
        push_pull.param_name.clear();
        push_pull.old_param_value = 0.0;
        push_pull.old_translation = None;
        return;
    }

    // ── Compute extrusion / parameter distance ─────────────────────
    let denom = plane_normal.dot(ray_dir);
    if denom.abs() < 1e-6 {
        return; // ray parallel to face
    }
    let current_t = (plane_origin - ray_origin).dot(plane_normal) / denom;
    if current_t < 0.0 {
        return;
    }
    let extrusion_t: f64 = (current_t - push_pull.drag_start_t) as f64;

    // No snap during parametric drag — raw continuous movement.
    // For non-parametric mesh extrusion use 0.1 unit snapping only.
    let snapped = if push_pull.is_parametric {
        // Round to 2 decimal places for display smoothness
        (extrusion_t * 100.0).round() / 100.0
    } else {
        // Snap non-parametric mesh extrusion to 0.1 unit increments
        (extrusion_t / 0.1).round() * 0.1
    };

    if (snapped - push_pull.accumulated_distance).abs() < 0.001 {
        return; // micro-update guard
    }

    if push_pull.is_parametric {
        // Parametric push-pull: mutate the parameter directly.
        let raw_new = (push_pull.old_param_value + snapped).max(0.01); // minimum 0.01
        let actual_delta = raw_new - push_pull.old_param_value; // clamped real change

        let scene = &mut state.document.recipe.scene;
        kpe_parametric::set_node_parameter(scene, &node_id, &push_pull.param_name, raw_new);
        
        // Base translation on the ACTUAL clamped delta (not the raw snapped)
        // to keep the opposite face stationary even when hitting the minimum.
        let shift = actual_delta / 2.0;
        let old_tx = push_pull.old_translation.unwrap_or([0.0; 3]);
        
        let matrices = kpe_geometry::evaluator::compute_world_matrices(scene, glam::DMat4::IDENTITY);
        let world_mat = matrices.get(&node_id).map(|m| dmat4_to_mat4(*m)).unwrap_or(Mat4::IDENTITY);
        let inv = world_mat.inverse();
        let face_normal_world = Vec3::new(face.normal.x as f32, face.normal.y as f32, face.normal.z as f32);
        let face_normal_local = inv.transform_vector3(face_normal_world).normalize();

        let new_tx = [
            old_tx[0] + face_normal_local.x as f64 * shift,
            old_tx[1] + face_normal_local.y as f64 * shift,
            old_tx[2] + face_normal_local.z as f64 * shift,
        ];
        kpe_parametric::commands::set_node_translation(scene, &node_id, new_tx);

        // Re-evaluate the node to rebuild its mesh.
        state.document.evaluate_node(&node_id);
        state.mark_dirty();
    } else {
        // Standard mesh extrusion from the original mesh each frame.
        if let Some(orig) = &push_pull.original_mesh {
            let result = extrude_face(orig, &face, snapped);
            state.document.evaluated.meshes.insert(node_id.clone(), result);
            state.mark_dirty();
        }
    }

    push_pull.accumulated_distance = snapped;
}

/// Draw the selected face boundary as a turquoise wireframe highlight.
pub fn draw_selected_face_system(
    mut gizmos: Gizmos,
    state: Res<AppState>,
    push_pull: Res<PushPullToolState>,
    editor: Res<crate::sketch_editor::SketchEditorState>,
) {
    if editor.active {
        return;
    }

    let node_id = match &push_pull.selected_node {
        Some(n) => n,
        None => return,
    };
    let face = match &push_pull.cached_face {
        Some(f) => f,
        None => return,
    };

    // Don't draw during drag (mesh updates visually).
    if push_pull.dragging {
        return;
    }

    let tri_mesh = match state.document.evaluated.meshes.get(node_id) {
        Some(m) => m,
        None => return,
    };

    // Draw the face boundary loop.
    let highlight = Color::srgb(0.1, 0.6, 1.0);
    for &(a, b) in &face.boundary_loop {
        let va = Vec3::new(
            tri_mesh.vertices[a as usize][0] as f32,
            tri_mesh.vertices[a as usize][1] as f32,
            tri_mesh.vertices[a as usize][2] as f32,
        );
        let vb = Vec3::new(
            tri_mesh.vertices[b as usize][0] as f32,
            tri_mesh.vertices[b as usize][1] as f32,
            tri_mesh.vertices[b as usize][2] as f32,
        );
        
        // Thicker highlight by drawing close offsets
        gizmos.line(va, vb, highlight);
        
        let center = (va + vb) / 2.0;
        let dir = (vb - va).normalize();
        let cross = center.cross(dir).normalize_or_zero() * 0.005;
        gizmos.line(va + cross, vb + cross, highlight);
    }
}
