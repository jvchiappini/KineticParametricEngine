use bevy::prelude::*;
use crate::{gizmos, sketch_editor};

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
) {
    if editor.active { return; }
    let size = 10.0;
    let step = 1.0;
    let half = size / 2.0;
    let steps = (size / step) as i32;
    let dark = Color::srgb(0.2, 0.2, 0.2);
    let light = Color::srgb(0.35, 0.35, 0.35);
    for i in -steps..=steps {
        let v = i as f32 * step;
        let color = if i == 0 { light } else { dark };
        gizmos.line(Vec3::new(v, 0.0, -half), Vec3::new(v, 0.0, half), color);
        gizmos.line(Vec3::new(-half, 0.0, v), Vec3::new(half, 0.0, v), color);
    }
}

fn axis_indicator(
    mut gizmos: Gizmos,
    editor: Res<sketch_editor::SketchEditorState>,
) {
    if editor.active { return; }
    let len = 1.5;
    gizmos.line(Vec3::ZERO, Vec3::new(len, 0.0, 0.0), Color::srgb(1.0, 0.0, 0.0));
    gizmos.line(Vec3::ZERO, Vec3::new(0.0, len, 0.0), Color::srgb(0.0, 1.0, 0.0));
    gizmos.line(Vec3::ZERO, Vec3::new(0.0, 0.0, len), Color::srgb(0.0, 0.0, 1.0));
    gizmos.sphere(Vec3::new(len, 0.0, 0.0), 0.06, Color::srgb(1.0, 0.0, 0.0));
    gizmos.sphere(Vec3::new(0.0, len, 0.0), 0.06, Color::srgb(0.0, 1.0, 0.0));
    gizmos.sphere(Vec3::new(0.0, 0.0, len), 0.06, Color::srgb(0.0, 0.0, 1.0));
}
