//! Desktop command helpers and re-exports from `kpe-parametric`.
//!
//! This module re-exports commonly used items for backward compatibility
//! and keeps clipboard operations that reference `AppState`.

pub use kpe_parametric::{
    CommandHistory,
    SetParameterCommand,
    AddFeatureCommand,
    DeleteFeatureCommand,
    SetSketchCommand,
    AddJointCommand,
    SetJointValueCommand,
    find_node,
    find_parent,
    find_parent_mut,
    collect_ids,
    reassign_ids,
    add_child,
    remove_child,
};

use kpe_parametric::commands::{
    find_node as kpe_find_node, find_parent as kpe_find_parent,
    collect_ids as kpe_collect_ids, reassign_ids as kpe_reassign_ids,
    AddFeatureCommand, DeleteFeatureCommand,
};
use kpe_schema::geometry::GeometryNode;

/// Copy the selected node to the clipboard.
pub fn copy_selected(state: &mut crate::app::AppState) {
    let selected = match &state.document.selection {
        Some(id) => id.clone(),
        None => return,
    };
    let node = kpe_find_node(&state.document.recipe.scene, &selected);
    if let Some(n) = node {
        state.clipboard = Some(n.clone());
    }
}

/// Cut the selected node (copy + delete) and store in clipboard.
pub fn cut_selected(state: &mut crate::app::AppState) {
    let selected = match &state.document.selection {
        Some(id) => id.clone(),
        None => return,
    };
    let cloned = match kpe_find_node(&state.document.recipe.scene, &selected).cloned() {
        Some(n) => n,
        None => return,
    };
    let parent = kpe_find_parent(&state.document.recipe.scene, &selected);
    let parent_id = parent.map(|p| p.id.clone());

    let mut cmd = DeleteFeatureCommand {
        parent_id: parent_id.unwrap_or_default(),
        node: cloned.clone(),
    };
    let mut gs = state.document.to_scene();
    cmd.execute(&mut gs);
    state.document.apply_scene(gs);
    state.history.undo_stack.push(Box::new(cmd));
    state.clipboard = Some(cloned);
    state.document.selection = None;
    state.mark_dirty();
}

/// Paste the clipboard contents into the scene.
pub fn paste_clipboard(state: &mut crate::app::AppState) {
    let clip = match &state.clipboard {
        Some(c) => c.clone(),
        None => return,
    };

    let selected = state.document.selection.clone();
    let target_id = if let Some(ref id) = selected {
        let is_valid = kpe_find_node(&state.document.recipe.scene, id).is_some();
        if is_valid {
            kpe_find_parent(&state.document.recipe.scene, id)
                .map_or("Root".to_string(), |p| p.id.clone())
        } else {
            "Root".to_string()
        }
    } else {
        "Root".to_string()
    };

    let mut existing_ids = Vec::new();
    kpe_collect_ids(&state.document.recipe.scene, &mut existing_ids);

    let mut pasted = clip;
    kpe_reassign_ids(&mut pasted, &mut existing_ids);

    let cmd = AddFeatureCommand {
        parent_id: target_id,
        node: pasted,
    };
    let mut gs = state.document.to_scene();
    state.history.execute(Box::new(cmd), &mut gs);
    state.document.apply_scene(gs);
    state.mark_dirty();
}
