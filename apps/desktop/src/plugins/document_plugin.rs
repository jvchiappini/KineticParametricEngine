use bevy::prelude::*;
use crate::{app, commands, sync, io};
use std::time::Duration;

pub struct DocumentPlugin;

impl Plugin for DocumentPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(AutoSaveTimer(Timer::new(Duration::from_secs(120), TimerMode::Repeating)))
            .insert_resource(sync::MeshCache::default())
            .add_systems(Startup, sync::setup_scene)
            .add_systems(Update, sync::sync_meshes)
            .add_systems(Update, keyboard_shortcuts)
            .add_systems(Update, auto_save_system)
            .add_systems(Update, update_window_title);
    }
}

fn keyboard_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<app::AppState>,
    editor: Res<crate::sketch_editor::SketchEditorState>,
) {
    if editor.active { return; }

    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
        commands::delete_selected_nodes(&mut *state);
        return;
    }

    if !ctrl { return; }

    if keys.just_pressed(KeyCode::KeyZ) && !keys.pressed(KeyCode::ShiftLeft) && !keys.pressed(KeyCode::ShiftRight) {
        state.undo();
    } else if keys.just_pressed(KeyCode::KeyZ) {
        state.redo();
    } else if keys.just_pressed(KeyCode::KeyY) {
        state.redo();
    } else if keys.just_pressed(KeyCode::KeyC) {
        commands::copy_selected(&mut *state);
    } else if keys.just_pressed(KeyCode::KeyX) {
        commands::cut_selected(&mut *state);
    } else if keys.just_pressed(KeyCode::KeyV) {
        commands::paste_clipboard(&mut *state);
    } else if keys.just_pressed(KeyCode::KeyA) {
        let mut all_ids = Vec::new();
        kpe_parametric::commands::collect_ids(&state.document.recipe.scene, &mut all_ids);
        state.document.multi_selection = all_ids;
        state.document.selection = None;
    } else if keys.just_pressed(KeyCode::KeyD) {
        commands::duplicate_selected(&mut *state);
    } else if keys.just_pressed(KeyCode::KeyS) {
        let _ = io::save_dialog(&state.document);
    }
}

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
