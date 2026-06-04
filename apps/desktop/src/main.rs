mod app;
mod build_tool;
mod camera;
mod commands;
mod document;
mod feature_commands;
mod gizmos;
mod io;
mod plugins;
mod tools;
mod push_pull_system;
mod sketch_editor;
mod sketch_render;
mod sync;
mod ui;
mod units;

use bevy::{
    core_pipeline::prepass::{DepthPrepass, NormalPrepass},
    pbr::ScreenSpaceAmbientOcclusion,
    prelude::*,
    render::view::Msaa,
};

use bevy_egui::EguiPlugin;

fn main() {
    // On Windows, prefer DX12 to avoid Vulkan validation-layer spam
    // (SPIR-V layout warnings and swapchain semaphore reuse false
    //  positives that are harmless but flood the console).
    #[cfg(target_os = "windows")]
    std::env::set_var("WGPU_BACKEND", "dx12");

    // Suppress verbose wgpu / DX12 logs — all are internal Bevy noise
    // not actionable by the user (CreateSampler, EMPTY_DISPATCH, etc.).
    tracing_subscriber::fmt()
        .with_env_filter(
            "wgpu_hal=error,wgpu_core=error,bevy_render=warn"
        )
        .init();

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
        .insert_resource(push_pull_system::PushPullState::default())
        .insert_resource(tools::PushPullToolState::default())
        .insert_resource(build_tool::BuildToolState::default())
        .add_systems(Startup, setup)
        .add_systems(Update, camera::orbit_camera_system)
        .add_systems(Update, view_preset_handler)
        .add_systems(Update, build_tool::tool_shortcut_system)
        .add_systems(Update, tools::select::viewport_selection)
        .add_systems(Update, (
            // New face-based push-pull.
            tools::push_pull::face_pick_system,
            tools::push_pull::push_pull_drag_system,
            tools::push_pull::draw_selected_face_system,
        ).chain())
        .add_systems(Update, (
            tools::rectangle::rect_tool_system,
            tools::circle::circle_tool_system,
            tools::line::line_tool_system,
            tools::line::line_preview_system,
            tools::move_tool::move_tool_system,
            tools::eraser::eraser_tool_system,
        ).chain())
        .add_systems(Update, build_tool::tool_render_system)
        .add_systems(Update, build_tool::vertical_toolbar_ui_system)
        .add_systems(Update, ui::measurements::measurements_ui_system)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 10_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4) * Quat::from_rotation_y(std::f32::consts::FRAC_PI_4)),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-180.0, 150.0, 180.0).looking_at(Vec3::ZERO, Vec3::Y),
        camera::OrbitCamera::default(),
        Msaa::Off,
        DepthPrepass,
        NormalPrepass,
        ScreenSpaceAmbientOcclusion::default(),
    ));
}

// ── View preset handler ──────────────────────

fn view_preset_handler(
    mut state: ResMut<app::AppState>,
    mut cameras: Query<&mut camera::OrbitCamera>,
) {
    let Some(view) = state.pending_view_preset.take() else { return };
    let Ok(mut cam) = cameras.get_single_mut() else { return };
    match view {
        1 => { cam.target = Vec3::ZERO; cam.distance = 1500.0; cam.yaw = 0.0; cam.pitch = 0.0; }
        2 => { cam.target = Vec3::ZERO; cam.distance = 1500.0; cam.yaw = 0.0; cam.pitch = std::f32::consts::FRAC_PI_2 - 0.01; }
        3 => { cam.target = Vec3::ZERO; cam.distance = 1500.0; cam.yaw = std::f32::consts::FRAC_PI_2; cam.pitch = 0.0; }
        4 => { cam.target = Vec3::ZERO; cam.distance = 1500.0; cam.yaw = 0.4; cam.pitch = 0.4; }
        _ => {}
    }
}
