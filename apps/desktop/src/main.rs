mod app;
mod camera;
mod commands;
mod document;
mod feature_commands;
mod gizmos;
mod io;
mod plugins;
mod sketch_editor;
mod sketch_render;
mod sync;
mod ui;

use bevy::{
    core_pipeline::prepass::{DepthPrepass, NormalPrepass},
    pbr::ScreenSpaceAmbientOcclusion,
    prelude::*,
    render::primitives::Aabb,
    render::view::Msaa,
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
        .add_plugins(plugins::DocumentPlugin)
        .add_plugins(plugins::GizmoPlugin)
        .add_plugins(plugins::UiPlugin)
        .add_plugins(plugins::SketchEditorPlugin)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 200.0,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, camera::orbit_camera_system)
        .add_systems(Update, view_preset_handler)
        .add_systems(Update, viewport_selection)
        .run();
}

fn setup(mut commands: Commands) {
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
        Msaa::Off,
        DepthPrepass,
        NormalPrepass,
        ScreenSpaceAmbientOcclusion::default(),
    ));
}

// ── Viewport selection via ray-AABB ──────────────

fn viewport_selection(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<camera::OrbitCamera>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    meshes: Query<(Entity, &sync::MeshNodeId, &Aabb, &GlobalTransform)>,
    mut state: ResMut<app::AppState>,
    editor: Res<sketch_editor::SketchEditorState>,
) {
    if editor.active { return; }
    if !mouse.just_pressed(MouseButton::Left) { return; }
    if mouse.pressed(MouseButton::Right) || mouse.pressed(MouseButton::Middle) { return; }

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
    let Ok(ray) = cam.viewport_to_world(cam_transform, cursor) else { return };
    let ray_origin = ray.origin;
    let ray_dir = ray.direction.as_vec3();

    let viewport_size = &window.resolution;
    if cursor.x < 220.0 || cursor.x > viewport_size.width() - 280.0 { return; }

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

// ── View preset handler ──────────────────────

fn view_preset_handler(
    mut state: ResMut<app::AppState>,
    mut cameras: Query<&mut camera::OrbitCamera>,
) {
    let Some(view) = state.pending_view_preset.take() else { return };
    let Ok(mut cam) = cameras.get_single_mut() else { return };
    match view {
        1 => { cam.target = Vec3::ZERO; cam.distance = 12.0; cam.yaw = 0.0; cam.pitch = 0.0; }
        2 => { cam.target = Vec3::ZERO; cam.distance = 12.0; cam.yaw = 0.0; cam.pitch = std::f32::consts::FRAC_PI_2 - 0.01; }
        3 => { cam.target = Vec3::ZERO; cam.distance = 12.0; cam.yaw = std::f32::consts::FRAC_PI_2; cam.pitch = 0.0; }
        4 => { cam.target = Vec3::ZERO; cam.distance = 10.0; cam.yaw = 0.4; cam.pitch = 0.4; }
        _ => {}
    }
}
