//! Move tool — click node → drag to translate.
//!
//! Phase flow:
//!   Idle + click → Dragging { start_world, node_id, old_translation }
//!   Dragging + mouse held → live translation update via direct mutation
//!   Dragging + release → push MoveNodeTransformCommand to undo stack

use bevy::prelude::*;
use bevy::render::primitives::Aabb;

use crate::{app::AppState, sync::MeshNodeId, tools::core::*};

/// Move tool system.
///
/// Activated when `BuildTool::Move` is the active tool (`[M]` key).
pub fn move_tool_system(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<crate::camera::OrbitCamera>>,
    mouse: Res<ButtonInput<MouseButton>>,
    meshes_query: Query<(&MeshNodeId, &GlobalTransform, &Aabb)>,
    mut state: ResMut<AppState>,
    mut tool_state: ResMut<BuildToolState>,
    editor: Res<crate::sketch_editor::SketchEditorState>,
) {
    if editor.active {
        return;
    }
    if tool_state.active_tool != BuildTool::Move {
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

    // Get camera forward for the drag plane.
    let (_, cam_tr) = match cameras.get_single() {
        Ok(c) => c,
        _ => return,
    };
    let forward: Vec3 = (-cam_tr.forward()).into();

    match tool_state.phase.clone() {
        ToolPhase::Idle => {
            if !mouse.just_pressed(MouseButton::Left) {
                return;
            }

            if let Some((node_id, hit)) = pick_node_aabb(ray_o, ray_d, &meshes_query) {
                let old_t = kpe_parametric::get_node_translation(
                    &state.document.recipe.scene,
                    &node_id,
                );
                tool_state.phase = ToolPhase::Dragging {
                    start_world: hit,
                    node_id,
                    old_translation: old_t,
                };
                tool_state.inference_text = "Moving…".into();
            }
        }
        ToolPhase::Dragging {
            start_world,
            node_id,
            old_translation,
        } => {
            // ── Release: finalise and push undo command ─────────────
            if mouse.just_released(MouseButton::Left) {
                let new_translation = kpe_parametric::get_node_translation(
                    &state.document.recipe.scene,
                    &node_id,
                );
                if let Some(new_t) = new_translation {
                    let cmd = Box::new(kpe_parametric::MoveNodeTransformCommand {
                        node_id: node_id.clone(),
                        old_translation,
                        new_translation: new_t,
                    });
                    state.history.undo_stack.push(cmd);
                    state.history.redo_stack.clear();
                    if state.history.undo_stack.len() > state.history.max_undo {
                        state.history.undo_stack.remove(0);
                    }
                }
                state.document.selection = Some(node_id.clone());
                tool_state.phase = ToolPhase::Idle;
                tool_state.inference_text.clear();
                return;
            }

            // ── Live drag: update translation in real-time ──────────
            if mouse.pressed(MouseButton::Left) {
                let plane = ConstructionPlane::from_normal(start_world, forward);
                let Some(cursor_world) = plane.intersect_ray(ray_o, ray_d) else {
                    return;
                };
                let delta = (cursor_world - start_world).as_dvec3();
                let base = old_translation.unwrap_or([0.0; 3]);
                let new_t: [f64; 3] = [
                    base[0] + delta.x,
                    base[1] + delta.y,
                    base[2] + delta.z,
                ];

                kpe_parametric::set_node_translation(
                    &mut state.document.recipe.scene,
                    &node_id,
                    new_t,
                );
                state.document.evaluate_node(&node_id);
                state.mark_dirty();
            }
        }
        _ => {
            tool_state.phase = ToolPhase::Idle;
        }
    }
}
