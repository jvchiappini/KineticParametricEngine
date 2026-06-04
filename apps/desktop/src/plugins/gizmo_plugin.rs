use bevy::prelude::*;
use crate::{gizmos, sketch_editor};
use crate::camera::OrbitCamera;

pub struct GizmoPlugin;

impl Plugin for GizmoPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(gizmos::GizmoState::default())
            .add_systems(Update, viewport_grid)
            .add_systems(Update, axis_indicator)
            .add_systems(Update, gizmos::gizmo_interaction_system)
            .add_systems(Update, gizmos::gizmo_render_system);
    }
}

fn viewport_grid(
    mut gizmos: Gizmos,
    editor: Res<sketch_editor::SketchEditorState>,
    cameras: Query<&OrbitCamera>,
) {
    if editor.active { return; }

    let cam_dist = cameras.get_single().map(|c| c.distance).unwrap_or(300.0);
    let (major, minor) = crate::units::grid_steps(cam_dist);

    // grid extends to 4× the camera radius so it always fills the viewport
    let half = (cam_dist * 4.0).max(major * 20.0);

    // Minor grid lines
    let minor_color = Color::srgba(0.25, 0.25, 0.25, 0.6);
    let major_color = Color::srgba(0.45, 0.45, 0.45, 0.8);
    let axis_color_x = Color::srgba(0.7, 0.2, 0.2, 0.9);
    let axis_color_z = Color::srgba(0.2, 0.2, 0.7, 0.9);

    let mut v = -(half / minor).ceil() as i32 * minor as i32;
    let end = (half / minor).ceil() as i32 * minor as i32;
    while v <= end {
        let fv = v as f32;
        let is_major = (fv / major).abs().fract() < 0.01;
        let is_x_axis = fv.abs() < minor * 0.5;
        let color = if is_x_axis {
            axis_color_x
        } else if is_major {
            major_color
        } else {
            minor_color
        };
        // lines along X
        gizmos.line(Vec3::new(-half, 0.0, fv), Vec3::new(half, 0.0, fv), color);
        v += minor as i32;
    }

    let mut v = -(half / minor).ceil() as i32 * minor as i32;
    while v <= end {
        let fv = v as f32;
        let is_major = (fv / major).abs().fract() < 0.01;
        let is_z_axis = fv.abs() < minor * 0.5;
        let color = if is_z_axis {
            axis_color_z
        } else if is_major {
            major_color
        } else {
            minor_color
        };
        // lines along Z
        gizmos.line(Vec3::new(fv, 0.0, -half), Vec3::new(fv, 0.0, half), color);
        v += minor as i32;
    }
}

fn axis_indicator(
    mut gizmos: Gizmos,
    editor: Res<sketch_editor::SketchEditorState>,
    cameras: Query<&OrbitCamera>,
) {
    if editor.active { return; }
    let cam_dist = cameras.get_single().map(|c| c.distance).unwrap_or(300.0);
    let len = cam_dist * 0.08; // scale with zoom so always visible
    gizmos.line(Vec3::ZERO, Vec3::new(len, 0.0, 0.0), Color::srgb(1.0, 0.15, 0.15));
    gizmos.line(Vec3::ZERO, Vec3::new(0.0, len, 0.0), Color::srgb(0.15, 1.0, 0.15));
    gizmos.line(Vec3::ZERO, Vec3::new(0.0, 0.0, len), Color::srgb(0.15, 0.15, 1.0));
}

