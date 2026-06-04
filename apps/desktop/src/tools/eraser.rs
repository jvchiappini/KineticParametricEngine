//! Eraser tool — click on a mesh entity to delete it.
//!
//! Uses ray‑AABB picking to find the node under the cursor, then
//! creates a `DeleteFeatureCommand` to remove it from the scene graph.

use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use kpe_geometry::evaluator::{find_node, find_parent};
use kpe_parametric::commands::DeleteFeatureCommand;

use crate::{app::AppState, sync::MeshNodeId, tools::core::*};

/// Eraser tool system.
///
/// Activated when `BuildTool::Eraser` is the active tool (`[E]` key).
pub fn eraser_tool_system(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<crate::camera::OrbitCamera>>,
    mouse: Res<ButtonInput<MouseButton>>,
    meshes_query: Query<(&MeshNodeId, &GlobalTransform, &Aabb)>,
    mut state: ResMut<AppState>,
    tool_state: Res<BuildToolState>,
    editor: Res<crate::sketch_editor::SketchEditorState>,
) {
    if editor.active {
        return;
    }
    if tool_state.active_tool != BuildTool::Eraser {
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Some((ray_o, ray_d)) = get_ray(&windows, &cameras) else {
        return;
    };
    let window = match windows.get_single() {
        Ok(w) => w,
        _ => return,
    };
    let cursor = match window.cursor_position() {
        Some(c) => c,
        None => return,
    };
    if !in_viewport(cursor, window) {
        return;
    }

    if let Some((node_id, _hit)) = pick_node_aabb(ray_o, ray_d, &meshes_query) {
        // Find the node and its parent in the scene tree.
        let scene = &state.document.recipe.scene;
        if let Some(node) = find_node(scene, &node_id) {
            let parent_id = find_parent(scene, &node_id)
                .map(|p| p.id.clone())
                .unwrap_or_else(|| "Root".to_string());
            let cmd = DeleteFeatureCommand {
                parent_id,
                node: node.clone(),
            };
            state.execute(Box::new(cmd));
            state.document.selection = None;
        }
    }
}
