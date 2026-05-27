mod app;
mod camera;
mod commands;
mod document;
mod gizmos;
mod io;
mod sync;
mod ui;

use bevy::{
    core_pipeline::prepass::{DepthPrepass, NormalPrepass},
    pbr::ScreenSpaceAmbientOcclusion,
    prelude::*,
};
use bevy_egui::EguiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "KPE Desktop".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 200.0,
        })
        .insert_resource(app::AppState::new())
        .insert_resource(sync::MeshCache::default())
        .insert_resource(gizmos::GizmoState::default())
        .add_systems(Startup, setup)
        .add_systems(Startup, sync::setup_scene)
        .add_systems(Update, app::ui_system)
        .add_systems(Update, sync::sync_meshes)
        .add_systems(Update, camera::orbit_camera_system)
        .add_systems(Update, gizmos::gizmo_interaction_system)
        .add_systems(Update, gizmos::gizmo_render_system)
        .run();
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 8_000_000.0,
            ..default()
        },
        Transform::from_xyz(3.0, 8.0, 5.0),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-6.0, 5.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        camera::OrbitCamera::default(),
        DepthPrepass,
        NormalPrepass,
        ScreenSpaceAmbientOcclusion::default(),
    ));
}
