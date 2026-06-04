//! Select tool — ray‑AABB node picking.
//!
//! Activated when `BuildTool::Select` is the active tool (Space key).
//! Supports single‑click selection and Ctrl+click multi‑selection.

use bevy::prelude::*;
use bevy::render::primitives::Aabb;

use crate::{app::AppState, camera, sync::MeshNodeId, tools::core::*};

/// Ray‑AABB node selection system.
///
/// Only responds when `BuildTool::Select` is active and the sketch editor
/// is not open.  Ctrl+click toggles multi‑selection.
pub fn viewport_selection(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<camera::OrbitCamera>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    meshes: Query<(Entity, &MeshNodeId, &Aabb, &GlobalTransform)>,
    mut state: ResMut<AppState>,
    editor: Res<crate::sketch_editor::SketchEditorState>,
    tool_state: Res<BuildToolState>,
) {
    if editor.active {
        return;
    }
    if !is_select_active(&tool_state) {
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    if mouse.pressed(MouseButton::Right) || mouse.pressed(MouseButton::Middle) {
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

    if !in_viewport(cursor, window) {
        return;
    }

    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    let mut best: Option<(f32, String)> = None;

    for (_entity, node_id, aabb, transform) in &meshes {
        let model: Mat4 = transform.compute_matrix();
        let inv: Mat4 = model.inverse();
        let local_origin: Vec3 = inv.transform_point3(ray_origin);
        let local_dir: Vec3 = inv.transform_vector3(ray_dir);
        let dir_rcp: Vec3 = local_dir.recip();

        let min: Vec3 = aabb.min().into();
        let max: Vec3 = aabb.max().into();
        let t1: Vec3 = (min - local_origin) * dir_rcp;
        let t2: Vec3 = (max - local_origin) * dir_rcp;
        let tmin: Vec3 = t1.min(t2);
        let tmax: Vec3 = t1.max(t2);
        let near: f32 = tmin.x.max(tmin.y).max(tmin.z);
        let far: f32 = tmax.x.min(tmax.y).min(tmax.z);

        if near <= far && far >= 0.0 {
            let hit: f32 = if near >= 0.0 { near } else { far };
            if best.as_ref().map_or(true, |(d, _)| hit < *d) {
                best = Some((hit, node_id.0.clone()));
            }
        }
    }

    if let Some((_, id)) = best {
        if ctrl {
            if state.document.selection.as_deref() == Some(&id) {
                state.document.selection = None;
            } else {
                state.document.selection = Some(id);
            }
        } else {
            state.document.selection = Some(id);
        }
    }
}
