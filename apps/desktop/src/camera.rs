use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::math::Vec3;
use crate::sync::MeshNodeId;
use bevy::render::primitives::Aabb;

#[derive(Component)]
pub struct OrbitCamera {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub fov_y: f32,
    pub sensitivity: f32,
    pub zoom_speed: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.4,
            distance: 10.0,
            fov_y: 45.0_f32.to_radians(),
            sensitivity: 0.005,
            zoom_speed: 0.1,
        }
    }
}

pub fn orbit_camera_system(
    mut query: Query<(&mut Transform, &mut OrbitCamera)>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut motion_evr: EventReader<MouseMotion>,
    mut scroll_evr: EventReader<MouseWheel>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mesh_query: Query<&Aabb, With<MeshNodeId>>,
) {
    let (mut transform, mut camera) = match query.get_single_mut() {
        Ok(q) => q,
        _ => return,
    };

    let mut delta = Vec2::ZERO;
    for ev in motion_evr.read() {
        delta += ev.delta;
    }

    let mut scroll = 0.0;
    for ev in scroll_evr.read() {
        scroll += ev.y;
    }

    // Right mouse button: orbit
    if mouse_buttons.pressed(MouseButton::Right) {
        camera.yaw -= delta.x * camera.sensitivity;
        camera.pitch += delta.y * camera.sensitivity;
        camera.pitch = camera.pitch.clamp(-1.5, 1.5);
    }

    // Middle mouse button: pan
    if mouse_buttons.pressed(MouseButton::Middle) {
        let right = transform.right();
        let up = transform.up();
        let pan_speed = camera.distance * 0.002;
        camera.target -= right * delta.x * pan_speed;
        camera.target += up * delta.y * pan_speed;
    }

    // Scroll: zoom
    if scroll != 0.0 {
        camera.distance *= 1.0 - scroll * camera.zoom_speed;
        camera.distance = camera.distance.clamp(0.5, 100.0);
    }

    // F: fit all (content bounding box)
    if keyboard.just_pressed(KeyCode::KeyF) {
        fit_all(&mut camera, &mesh_query);
    }

    // View presets: 1=Front, 2=Top, 3=Right, 4=Iso
    if keyboard.just_pressed(KeyCode::Digit1) {
        camera.target = Vec3::ZERO;
        camera.distance = 12.0;
        camera.yaw = 0.0;
        camera.pitch = 0.0;
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        camera.target = Vec3::ZERO;
        camera.distance = 12.0;
        camera.yaw = 0.0;
        camera.pitch = std::f32::consts::FRAC_PI_2 - 0.01;
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        camera.target = Vec3::ZERO;
        camera.distance = 12.0;
        camera.yaw = std::f32::consts::FRAC_PI_2;
        camera.pitch = 0.0;
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        camera.target = Vec3::ZERO;
        camera.distance = 10.0;
        camera.yaw = 0.4;
        camera.pitch = 0.4;
    }

    // Compute camera position from spherical coordinates
    let pitch_sin = camera.pitch.sin();
    let pitch_cos = camera.pitch.cos();
    let yaw_sin = camera.yaw.sin();
    let yaw_cos = camera.yaw.cos();

    let eye = camera.target + Vec3::new(
        camera.distance * pitch_cos * yaw_sin,
        camera.distance * pitch_sin,
        camera.distance * pitch_cos * yaw_cos,
    );

    transform.translation = eye;
    transform.look_at(camera.target, Vec3::Y);
}

fn fit_all(camera: &mut OrbitCamera, mesh_query: &Query<&Aabb, With<MeshNodeId>>) {
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);
    let mut any = false;
    for aabb in mesh_query.iter() {
        min = min.min(aabb.min().into());
        max = max.max(aabb.max().into());
        any = true;
    }
    if !any {
        camera.target = Vec3::ZERO;
        camera.distance = 10.0;
        return;
    }
    let center = (min + max) * 0.5;
    let size = max - min;
    let radius = size.length() * 0.5;
    camera.target = center;
    camera.distance = (radius + 2.0).max(2.0);
    camera.yaw = 0.4;
    camera.pitch = 0.4;
}
