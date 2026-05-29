use bevy::prelude::*;
use crate::app::AppState;
use crate::sketch_editor::SketchEditorState;
use kpe_schema::geometry::{GeometryNode, TransformOp};

const AXIS_LENGTH: f32 = 1.5;
const HIT_RADIUS: f32 = 0.2;

#[derive(Clone, PartialEq)]
pub enum GizmoInteraction {
    None,
    Dragging { axis: usize, offset: f32 },
}

impl Default for GizmoInteraction {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Resource, Default)]
pub struct GizmoState {
    pub interaction: GizmoInteraction,
    pub hovered_axis: Option<usize>,
    pub drag_origin: Vec3,
}

pub fn gizmo_render_system(
    mut gizmos: Gizmos,
    state: Res<AppState>,
    gizmo_state: Res<GizmoState>,
    sketch: Res<SketchEditorState>,
) {
    if sketch.active { return; }
    let sel = match &state.document.selection {
        Some(id) => id.clone(),
        None => return,
    };

    let origin = node_position(&state.document.recipe.scene, &sel).unwrap_or(Vec3::ZERO);

    let colors = [Color::srgb(1.0, 0.2, 0.2), Color::srgb(0.2, 1.0, 0.2), Color::srgb(0.2, 0.2, 1.0)];
    let axes = [Vec3::X, Vec3::Y, Vec3::Z];

    for i in 0..3 {
        let end = origin + axes[i] * AXIS_LENGTH;
        gizmos.line(origin, end, colors[i]);
        gizmos.sphere(end, 0.08, colors[i]);
    }

    if let Some(hi) = gizmo_state.hovered_axis {
        let end = origin + axes[hi] * AXIS_LENGTH;
        gizmos.line(origin, end, Color::WHITE);
        gizmos.sphere(end, 0.12, Color::WHITE);
    }
}

fn node_position(node: &GeometryNode, target: &str) -> Option<Vec3> {
    if node.id == target {
        return match &node.transform {
            Some(tf) => tf.translation.map(|t| Vec3::new(t[0] as f32, t[1] as f32, t[2] as f32)),
            None => Some(Vec3::ZERO),
        };
    }
    for child in &node.children {
        if let found @ Some(_) = node_position(child, target) {
            return found;
        }
    }
    None
}

pub fn gizmo_interaction_system(
    mut state: ResMut<AppState>,
    mut gizmo_state: ResMut<GizmoState>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    windows: Query<&Window>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    sketch: Res<SketchEditorState>,
) {
    if sketch.active { return; }
    let sel = match &state.document.selection.clone() {
        Some(id) => id.clone(),
        None => return,
    };

    let origin = node_position(&state.document.recipe.scene, &sel).unwrap_or(Vec3::ZERO);

    let (camera, camera_transform) = match camera_query.get_single() {
        Ok(q) => q,
        _ => return,
    };

    let window = match windows.get_single() {
        Ok(w) => w,
        _ => return,
    };

    let cursor_pos = match window.cursor_position() {
        Some(p) => p,
        _ => return,
    };

    let viewport_size = match camera.logical_viewport_size() {
        Some(s) => s,
        None => return,
    };

    let ray = screen_to_ray(cursor_pos, viewport_size, camera, camera_transform);
    let axes = [Vec3::X, Vec3::Y, Vec3::Z];

    let is_none = gizmo_state.interaction == GizmoInteraction::None;
    let is_dragging = matches!(gizmo_state.interaction, GizmoInteraction::Dragging { .. });

    if is_none {
        gizmo_state.hovered_axis = None;
        for i in 0..3 {
            let end = origin + axes[i] * AXIS_LENGTH;
            if ray_hits_segment(ray.0, ray.1, origin, end) {
                gizmo_state.hovered_axis = Some(i);
                break;
            }
        }

        if mouse_buttons.just_pressed(MouseButton::Left) {
            if let Some(axis) = gizmo_state.hovered_axis {
                let end = origin + axes[axis] * AXIS_LENGTH;
                let hit = closest_point_on_ray(ray.0, ray.1, origin, end);
                let dist = (hit - ray.0).length();
                let hit_point = ray.0 + ray.1 * dist;
                let proj = (hit_point - origin).dot(axes[axis]);
                gizmo_state.interaction = GizmoInteraction::Dragging { axis, offset: proj };
                gizmo_state.drag_origin = origin;
            }
        }
        return;
    }

    if is_dragging {
        let drag_origin = gizmo_state.drag_origin;

        if mouse_buttons.just_released(MouseButton::Left) {
            gizmo_state.interaction = GizmoInteraction::None;
            return;
        }

        if let GizmoInteraction::Dragging { axis, offset } = &mut gizmo_state.interaction {
            let end = drag_origin + axes[*axis] * AXIS_LENGTH;
            let hit = closest_point_on_ray(ray.0, ray.1, drag_origin, end);
            let dist = (hit - ray.0).length();
            let hit_point = ray.0 + ray.1 * dist;
            let proj = (hit_point - drag_origin).dot(axes[*axis]);
            let delta = proj - *offset;

            *offset = proj;

            if delta.abs() > 0.001 {
                apply_translation(&mut state.document.recipe.scene, &sel, axes[*axis] * delta);
                state.document.evaluate_node(&sel);
                state.mark_dirty();
            }
        }
    }
}

fn screen_to_ray(
    cursor: Vec2,
    _viewport: Vec2,
    camera: &Camera,
    transform: &GlobalTransform,
) -> (Vec3, Vec3) {
    let Ok(ray) = camera.viewport_to_world(transform, cursor) else {
        return (Vec3::ZERO, Vec3::Z);
    };
    (ray.origin, *ray.direction)
}

fn ray_hits_segment(ray_origin: Vec3, ray_dir: Vec3, a: Vec3, b: Vec3) -> bool {
    let hit = closest_point_on_ray(ray_origin, ray_dir, a, b);
    let dist = (hit - ray_origin).length();
    let point = ray_origin + ray_dir * dist;
    let closest = closest_point_on_segment(a, b, point);
    (point - closest).length() < HIT_RADIUS
}

fn closest_point_on_ray(ray_origin: Vec3, ray_dir: Vec3, a: Vec3, b: Vec3) -> Vec3 {
    let seg = b - a;
    let d = ray_dir.cross(seg);
    let denom = d.length_squared();
    if denom < 1e-10 {
        return ray_origin;
    }
    let t = (seg.cross(a - ray_origin)).dot(d) / denom;
    ray_origin + ray_dir * t.max(0.0)
}

fn closest_point_on_segment(a: Vec3, b: Vec3, p: Vec3) -> Vec3 {
    let ab = b - a;
    let t = (p - a).dot(ab) / ab.length_squared();
    let t = t.clamp(0.0, 1.0);
    a + ab * t
}

fn apply_translation(node: &mut GeometryNode, target: &str, delta: Vec3) {
    if node.id != target {
        for child in &mut node.children {
            apply_translation(child, target, delta);
        }
        return;
    }
    let tf = node.transform.get_or_insert_with(|| TransformOp {
        translation: Some([0.0, 0.0, 0.0]),
        rotation: None,
        scale: None,
    });
    let t = tf.translation.get_or_insert_with(|| [0.0, 0.0, 0.0]);
    t[0] += delta.x as f64;
    t[1] += delta.y as f64;
    t[2] += delta.z as f64;
}
