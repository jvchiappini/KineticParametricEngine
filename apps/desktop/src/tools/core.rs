//! Shared types and helpers for the SketchUp‑style tool system.
//!
//! This module provides the common infrastructure used by all tools:
//! `BuildTool` enum, `ConstructionPlane`, `ToolPhase`, `BuildToolState`,
//! ray‑picking helpers, and math utilities.

use bevy::prelude::*;
use bevy::render::primitives::Aabb;

use crate::{app::AppState, camera, sync::MeshNodeId};

// ---------------------------------------------------------------------------
// Tool Enum
// ---------------------------------------------------------------------------

/// Identifies the active build tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BuildTool {
    #[default]
    Select,
    Rectangle,
    Circle,
    Line,
    PushPull,
    Move,
    Eraser,
}

impl BuildTool {
    pub fn label(&self) -> &'static str {
        match self {
            BuildTool::Select => "Select",
            BuildTool::Rectangle => "Rectangle",
            BuildTool::Circle => "Circle",
            BuildTool::Line => "Line",
            BuildTool::PushPull => "Push/Pull",
            BuildTool::Move => "Move",
            BuildTool::Eraser => "Eraser",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            BuildTool::Select => "Sel",
            BuildTool::Rectangle => "Rect",
            BuildTool::Circle => "Circ",
            BuildTool::Line => "Line",
            BuildTool::PushPull => "P/P",
            BuildTool::Move => "Move",
            BuildTool::Eraser => "Erase",
        }
    }

    pub fn keyboard_key(&self) -> KeyCode {
        match self {
            BuildTool::Select => KeyCode::Space,
            BuildTool::Rectangle => KeyCode::KeyR,
            BuildTool::Circle => KeyCode::KeyC,
            BuildTool::Line => KeyCode::KeyL,
            BuildTool::PushPull => KeyCode::KeyP,
            BuildTool::Move => KeyCode::KeyM,
            BuildTool::Eraser => KeyCode::KeyE,
        }
    }
}

// ---------------------------------------------------------------------------
// Construction Plane
// ---------------------------------------------------------------------------

/// A plane used for placing 2D tool geometry in 3D space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConstructionPlane {
    pub origin: Vec3,
    pub normal: Vec3,
    pub u_axis: Vec3,
    pub v_axis: Vec3,
}

impl ConstructionPlane {
    pub fn ground() -> Self {
        Self {
            origin: Vec3::ZERO,
            normal: Vec3::Y,
            u_axis: Vec3::X,
            v_axis: Vec3::Z,
        }
    }

    pub fn project(&self, point: Vec3) -> Vec3 {
        let to_point = point - self.origin;
        point - self.normal * to_point.dot(self.normal)
    }

    pub fn to_2d(&self, point: Vec3) -> Vec2 {
        let local = self.project(point) - self.origin;
        Vec2::new(local.dot(self.u_axis), local.dot(self.v_axis))
    }

    pub fn from_normal(origin: Vec3, normal: Vec3) -> Self {
        let u = if normal.y.abs() < 0.9 {
            normal.cross(Vec3::Y).normalize()
        } else {
            normal.cross(Vec3::Z).normalize()
        };
        let v = normal.cross(u).normalize();
        Self {
            origin,
            normal,
            u_axis: u,
            v_axis: v,
        }
    }

    /// Intersect a ray with this plane. Returns the world-space hit point.
    pub fn intersect_ray(&self, ray_origin: Vec3, ray_dir: Vec3) -> Option<Vec3> {
        let denom = self.normal.dot(ray_dir);
        if denom.abs() < 1e-6 {
            return None;
        }
        let t = (self.origin - ray_origin).dot(self.normal) / denom;
        if t < 0.0 {
            return None;
        }
        Some(ray_origin + ray_dir * t)
    }
}

// ---------------------------------------------------------------------------
// Tool Phase
// ---------------------------------------------------------------------------

/// State machine for multi‑click / drag tool operations.
#[derive(Debug, Clone, PartialEq)]
pub enum ToolPhase {
    Idle,
    Placing {
        p1_world: Vec3,
        plane: ConstructionPlane,
    },
    Polylining {
        prev_world: Vec3,
        plane: ConstructionPlane,
        poly_points: Vec<Vec3>,
    },
    Dragging {
        start_world: Vec3,
        node_id: String,
        old_translation: Option<[f64; 3]>,
    },
}

// ---------------------------------------------------------------------------
// Tool State Resource
// ---------------------------------------------------------------------------

/// Global Bevy resource holding the active tool and its phase.
#[derive(Resource, Debug, Clone)]
pub struct BuildToolState {
    pub active_tool: BuildTool,
    pub phase: ToolPhase,
    pub inference_text: String,
}

impl Default for BuildToolState {
    fn default() -> Self {
        Self {
            active_tool: BuildTool::Select,
            phase: ToolPhase::Idle,
            inference_text: String::new(),
        }
    }
}

/// Check if the Select tool is active.
pub fn is_select_active(state: &BuildToolState) -> bool {
    state.active_tool == BuildTool::Select
}

/// Check if the PushPull tool is active.
pub fn is_push_pull_active(state: &BuildToolState) -> bool {
    state.active_tool == BuildTool::PushPull
}

// ---------------------------------------------------------------------------
// Ray & Viewport Helpers
// ---------------------------------------------------------------------------

/// Get the mouse ray in world space from the active camera.
pub fn get_ray(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform), With<camera::OrbitCamera>>,
) -> Option<(Vec3, Vec3)> {
    let window = windows.get_single().ok()?;
    let cursor = window.cursor_position()?;
    let (cam, cam_transform) = cameras.get_single().ok()?;
    let Ok(ray) = cam.viewport_to_world(cam_transform, cursor) else {
        return None;
    };
    Some((ray.origin, *ray.direction))
}

/// Check whether the cursor is inside the viewport (outside UI panels).
pub fn in_viewport(cursor: Vec2, window: &Window) -> bool {
    cursor.x >= 220.0 && cursor.x <= window.resolution.width() - 280.0
}

// ---------------------------------------------------------------------------
// Math helpers
// ---------------------------------------------------------------------------

/// Möller–Trumbore ray‑triangle intersection.
pub fn ray_triangle_intersection(
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
    if t > 1e-5 {
        Some(t)
    } else {
        None
    }
}

/// Snap cursor ray to global axes stemming from `p1_world` (SketchUp style red/green/blue inference).
pub fn snap_to_axes(ray_o: Vec3, ray_d: Vec3, p1: Vec3, snap_threshold: f32) -> Option<Vec3> {
    let axes = [Vec3::X, Vec3::Y, Vec3::Z, -Vec3::X, -Vec3::Y, -Vec3::Z];
    let mut best: Option<(f32, Vec3)> = None;

    for &axis in &axes {
        let cross = ray_d.cross(axis);
        let denom = cross.length_squared();
        if denom < 1e-6 { // parallel
            continue;
        }

        let diff = p1 - ray_o;
        let t1 = (diff.cross(axis)).dot(cross) / denom;
        let t2 = (diff.cross(ray_d)).dot(cross) / denom;

        if t2 < 0.001 { // must point in the direction of the axis choice
            continue;
        }

        let pt_on_ray = ray_o + ray_d * t1;
        let pt_on_axis = p1 + axis * t2;

        let dist = (pt_on_ray - pt_on_axis).length();
        if dist < snap_threshold {
            if best.map_or(true, |(best_dist, _)| dist < best_dist) {
                best = Some((dist, pt_on_axis));
            }
        }
    }
    best.map(|(_, p)| p)
}

/// Convert a `glam::DMat4` to `bevy::Mat4`.
pub fn dmat4_to_mat4(m: glam::DMat4) -> Mat4 {
    Mat4::from_cols(
        Vec4::new(
            m.x_axis.x as f32,
            m.x_axis.y as f32,
            m.x_axis.z as f32,
            m.x_axis.w as f32,
        ),
        Vec4::new(
            m.y_axis.x as f32,
            m.y_axis.y as f32,
            m.y_axis.z as f32,
            m.y_axis.w as f32,
        ),
        Vec4::new(
            m.z_axis.x as f32,
            m.z_axis.y as f32,
            m.z_axis.z as f32,
            m.z_axis.w as f32,
        ),
        Vec4::new(
            m.w_axis.x as f32,
            m.w_axis.y as f32,
            m.w_axis.z as f32,
            m.w_axis.w as f32,
        ),
    )
}

// ---------------------------------------------------------------------------
// Scene picking helpers
// ---------------------------------------------------------------------------

/// Ray‑pick the closest face among all scene meshes.
///
/// Returns `(world_hit, world_normal, node_id)` for the closest triangle
/// intersected by the ray.
pub fn pick_face(
    ray_origin: Vec3,
    ray_dir: Vec3,
    meshes_query: &Query<(&MeshNodeId, &GlobalTransform, &Aabb)>,
    state: &AppState,
) -> Option<(Vec3, Vec3, String)> {
    let matrices = kpe_geometry::evaluator::compute_world_matrices(
        &state.document.recipe.scene,
        glam::DMat4::IDENTITY,
    );
    let mut best: Option<(f32, Vec3, Vec3, String)> = None;

    for (mesh_node_id, _tf, _aabb) in meshes_query.iter() {
        let nid = &mesh_node_id.0;
        let tri_mesh = state.document.evaluated.meshes.get(nid)?;
        if tri_mesh.triangles.is_empty() {
            continue;
        }
        let world = matrices
            .get(nid)
            .map(|m| dmat4_to_mat4(*m))
            .unwrap_or(Mat4::IDENTITY);
        let inv = world.inverse();
        let lo = inv.transform_point3(ray_origin);
        let ld = inv.transform_vector3(ray_dir).normalize();

        for tri in &tri_mesh.triangles {
            let v = &tri_mesh.vertices;
            let v0 = Vec3::new(
                v[tri[0] as usize][0] as f32,
                v[tri[0] as usize][1] as f32,
                v[tri[0] as usize][2] as f32,
            );
            let v1 = Vec3::new(
                v[tri[1] as usize][0] as f32,
                v[tri[1] as usize][1] as f32,
                v[tri[1] as usize][2] as f32,
            );
            let v2 = Vec3::new(
                v[tri[2] as usize][0] as f32,
                v[tri[2] as usize][1] as f32,
                v[tri[2] as usize][2] as f32,
            );
            if let Some(t) = ray_triangle_intersection(lo, ld, v0, v1, v2) {
                if best.as_ref().map_or(true, |(d, ..)| t < *d) {
                    let lh = lo + ld * t;
                    let wh = world.transform_point3(lh);
                    let ln = (v1 - v0).cross(v2 - v0).normalize();
                    let wn = world.transform_vector3(ln).normalize();
                    best = Some((t, wh, wn, nid.clone()));
                }
            }
        }
    }
    best.map(|(_, hp, n, id)| (hp, n, id))
}

/// Infer a construction plane from the geometry under the cursor.
///
/// 1. If a face is hit, use its position and normal.
/// 2. Fallback: intersect the ray with the ground plane (Y=0).
/// 3. Ultimate fallback: ground plane at origin.
pub fn infer_plane(
    ray_o: Vec3,
    ray_d: Vec3,
    meshes_query: &Query<(&MeshNodeId, &GlobalTransform, &Aabb)>,
    state: &AppState,
) -> ConstructionPlane {
    if let Some((hit, normal, _id)) = pick_face(ray_o, ray_d, meshes_query, state) {
        return ConstructionPlane::from_normal(hit, normal);
    }
    if let Some(hit) = ConstructionPlane::ground().intersect_ray(ray_o, ray_d) {
        return ConstructionPlane::from_normal(hit, Vec3::Y);
    }
    ConstructionPlane::ground()
}

/// Pick the closest mesh node under the cursor via ray‑AABB intersection.
///
/// Returns `(node_id, world_hit_point)`.
pub fn pick_node_aabb(
    ray_o: Vec3,
    ray_d: Vec3,
    meshes_query: &Query<(&MeshNodeId, &GlobalTransform, &Aabb)>,
) -> Option<(String, Vec3)> {
    let mut best: Option<(f32, String, Vec3)> = None;
    for (node_id, transform, aabb) in meshes_query.iter() {
        let model = transform.compute_matrix();
        let inv = model.inverse();
        let lo = inv.transform_point3(ray_o);
        let ld = inv.transform_vector3(ray_d);
        let dir_rcp = ld.recip();
        let min: Vec3 = aabb.min().into();
        let max: Vec3 = aabb.max().into();
        let t1 = (min - lo) * dir_rcp;
        let t2 = (max - lo) * dir_rcp;
        let tmin = t1.min(t2);
        let tmax = t1.max(t2);
        let near = tmin.x.max(tmin.y).max(tmin.z);
        let far = tmax.x.min(tmax.y).min(tmax.z);
        if near <= far && far >= 0.0 {
            let hit = if near >= 0.0 { near } else { far };
            if best.as_ref().map_or(true, |(d, ..)| hit < *d) {
                let world_hit = model.transform_point3(lo + ld * hit);
                best = Some((hit, node_id.0.clone(), world_hit));
            }
        }
    }
    best.map(|(_, id, hit)| (id, hit))
}
