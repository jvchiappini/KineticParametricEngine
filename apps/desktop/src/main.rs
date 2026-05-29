mod app;
mod camera;
mod commands;
mod document;
mod feature_commands;
mod gizmos;
mod io;
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
use std::time::Duration;

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
        .insert_resource(sketch_editor::SketchEditorState::new())
        .insert_resource(AutoSaveTimer(Timer::new(Duration::from_secs(120), TimerMode::Repeating)))
        .add_systems(Startup, setup)
        .add_systems(Startup, sync::setup_scene)
        .add_systems(Update, app::ui_system)
        .add_systems(Update, sync::sync_meshes)
        .add_systems(Update, camera::orbit_camera_system)
        .add_systems(Update, view_preset_handler)
        .add_systems(Update, keyboard_shortcuts)
        .add_systems(Update, viewport_selection)
        .add_systems(Update, auto_save_system)
        .add_systems(Update, update_window_title)
        .add_systems(Update, viewport_grid)
        .add_systems(Update, axis_indicator)
        .add_systems(Update, gizmos::gizmo_interaction_system)
        .add_systems(Update, gizmos::gizmo_render_system)
        .add_systems(Update, sketch_editor::check_enter_sketch_mode)
        .add_systems(Update, sketch_editor::sketch_input)
        .add_systems(Update, sketch_editor::render_sketch)
        .add_systems(Update, sketch_editor::render_sketch_wireframes)
        .add_systems(Update, sketch_editor::sketch_ui)
        .run();
}

pub fn keyboard_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<app::AppState>,
    editor: Res<sketch_editor::SketchEditorState>,
) {
    if editor.active { return; }

    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
        commands::delete_selected_nodes(&mut *state);
        return;
    }

    if !ctrl { return; }

    if keys.just_pressed(KeyCode::KeyZ) && !keys.pressed(KeyCode::ShiftLeft) && !keys.pressed(KeyCode::ShiftRight) {
        if let Some(mut cmd) = state.history.undo_stack.pop() {
            cmd.undo(&mut state.document);
            state.history.redo_stack.push(cmd);
            state.mark_dirty();
        }
    } else if keys.just_pressed(KeyCode::KeyZ) {
        if let Some(mut cmd) = state.history.redo_stack.pop() {
            cmd.execute(&mut state.document);
            state.history.undo_stack.push(cmd);
            state.mark_dirty();
        }
    } else if keys.just_pressed(KeyCode::KeyY) {
        if let Some(mut cmd) = state.history.redo_stack.pop() {
            cmd.execute(&mut state.document);
            state.history.undo_stack.push(cmd);
            state.mark_dirty();
        }
    } else if keys.just_pressed(KeyCode::KeyC) {
        commands::copy_selected(&mut *state);
    } else if keys.just_pressed(KeyCode::KeyX) {
        commands::cut_selected(&mut *state);
    } else if keys.just_pressed(KeyCode::KeyV) {
        commands::paste_clipboard(&mut *state);
    } else if keys.just_pressed(KeyCode::KeyA) {
        let mut all_ids = Vec::new();
        commands::collect_ids(&state.document.recipe.scene, &mut all_ids);
        state.document.multi_selection = all_ids;
        state.document.selection = None;
    } else if keys.just_pressed(KeyCode::KeyD) {
        commands::duplicate_selected(&mut *state);
    } else if keys.just_pressed(KeyCode::KeyS) {
        let _ = crate::io::save_dialog(&state.document);
    }
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

// ── Auto-save ──────────────────────────────────

#[derive(Resource)]
struct AutoSaveTimer(Timer);

fn auto_save_system(
    time: Res<Time>,
    mut timer: ResMut<AutoSaveTimer>,
    state: Res<app::AppState>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let path = dirs_data_local().join("kpe_autosave.kpe");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let json = serde_json::to_string(&state.document.recipe).unwrap_or_default();
        std::fs::write(&path, &json).ok();
    }
}

fn dirs_data_local() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("APPDATA").unwrap_or_else(|_| ".kpe".to_string()))
        .join("KPE")
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

// ── 3D viewport grid ──────────────────────────

pub fn viewport_grid(
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

// ── Axis indicator ────────────────────────────

pub fn axis_indicator(
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

// ── Window title ──────────────────────────────

fn update_window_title(
    state: Res<app::AppState>,
    mut windows: Query<&mut Window>,
) {
    let Ok(mut window) = windows.get_single_mut() else { return };
    let name = state.document.file_path.as_deref()
        .and_then(|p| std::path::Path::new(p).file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("Untitled");
    let modified = if state.document.is_modified { "*" } else { "" };
    window.title = format!("KPE Desktop - {}{}", name, modified);
}