//! Thin wrappers that bridge `AppState` to the parametric commands in
//! `kpe-parametric`.  All core logic lives in the crate; these functions
//! simply read selection state from the app and delegate.

use kpe_parametric::commands::features::{
    self as feature_cmds, ArrayParams, MirrorPlane,
};
use kpe_parametric::commands::find_node;
use kpe_parametric::commands::features::build_add_joint_command;

/// Duplicate the selected node.
pub fn duplicate_selected(state: &mut crate::app::AppState) {
    let selected = match &state.document.selection {
        Some(id) => id.clone(),
        None => return,
    };
    if let Some(cmd) = feature_cmds::build_duplicate_command(&state.document.to_scene(), &selected) {
        state.execute(cmd);
    }
}

/// Create an array of copies of the selected node.
pub fn array_selected(state: &mut crate::app::AppState, params: &ArrayParams) {
    let selected = match &state.document.selection {
        Some(id) => id.clone(),
        None => return,
    };
    if let Some(cmd) = feature_cmds::build_array_command(&state.document.to_scene(), &selected, params) {
        state.execute(cmd);
    }
}

/// Mirror the selected node across a plane.
pub fn mirror_selected(state: &mut crate::app::AppState, plane: &str) {
    let selected = match &state.document.selection {
        Some(id) => id.clone(),
        None => return,
    };
    let mp = match plane {
        "XY" => MirrorPlane::XY,
        "XZ" => MirrorPlane::XZ,
        "YZ" => MirrorPlane::YZ,
        _ => MirrorPlane::XY,
    };
    if let Some(cmd) = feature_cmds::build_mirror_command(&state.document.to_scene(), &selected, &mp) {
        state.execute(cmd);
    }
}

/// Wrap the selected node in a fillet operation.
pub fn add_fillet(state: &mut crate::app::AppState, radius: f64) {
    let selected = match &state.document.selection {
        Some(id) => id.clone(),
        None => return,
    };
    if let Some(cmd) = feature_cmds::build_fillet_command(&state.document.to_scene(), &selected, radius) {
        state.execute(cmd);
    }
}

/// Wrap the selected node in a chamfer operation.
pub fn add_chamfer(state: &mut crate::app::AppState, distance: f64) {
    let selected = match &state.document.selection {
        Some(id) => id.clone(),
        None => return,
    };
    if let Some(cmd) = feature_cmds::build_chamfer_command(&state.document.to_scene(), &selected, distance) {
        state.execute(cmd);
    }
}

/// Delete the selected node.
pub fn delete_selected_node(state: &mut crate::app::AppState) {
    let selected = match &state.document.selection {
        Some(id) => id.clone(),
        None => return,
    };
    if selected == "Root" {
        return;
    }
    if let Some(cmd) = feature_cmds::build_delete_command(&state.document.to_scene(), &selected) {
        state.execute(cmd);
        state.document.selection = None;
    }
}

/// Delete all selected nodes (multi-selection + selection).
pub fn delete_selected_nodes(state: &mut crate::app::AppState) {
    let mut ids = state.document.multi_selection.clone();
    if let Some(ref sel) = state.document.selection {
        if !ids.contains(sel) && sel != "Root" {
            ids.insert(0, sel.clone());
        }
    }
    if ids.is_empty() {
        return;
    }
    if let Some(cmd) = feature_cmds::build_delete_multi_command(&state.document.to_scene(), &ids) {
        state.execute(cmd);
    }
    state.document.selection = None;
    state.document.multi_selection.clear();
}

/// Wrap the selected node in a Compound group.
pub fn add_group(state: &mut crate::app::AppState, selected_id: &str) {
    if let Some(cmd) = feature_cmds::build_group_command(&state.document.to_scene(), selected_id) {
        state.execute(cmd);
        state.document.selection = Some(format!("Group_{}", selected_id));
    }
}

/// Wrap the selected node in an Assembly.
pub fn add_assembly(state: &mut crate::app::AppState, selected_id: &str) {
    if let Some(cmd) = feature_cmds::build_assembly_command(&state.document.to_scene(), selected_id) {
        state.execute(cmd);
        state.document.selection = Some(format!("Assembly_{}", selected_id));
    }
}

/// Add a joint between a parent and child node.
pub fn add_joint(state: &mut crate::app::AppState, selected_ids: &Option<String>) {
    let child_id = match selected_ids {
        Some(id) => id.clone(),
        None => return,
    };
    let parent = kpe_parametric::commands::find_parent(&state.document.recipe.scene, &child_id);
    let parent_id = parent.map(|p| p.id.clone()).unwrap_or_else(|| "Root".to_string());

    let joint_id = format!("Joint_{}", state.document.recipe.joints.len() + 1);
    let joint = kpe_schema::joint::Joint {
        id: joint_id,
        joint_type: state.new_joint_type.clone(),
        parent_id,
        child_id,
        pivot: state.new_joint_pivot,
        axis: state.new_joint_axis,
        limits: Some(kpe_schema::joint::JointLimits {
            min: -180.0,
            max: 180.0,
            damping: None,
            stiffness: None,
        }),
        current_value: 0.0,
    };
    state.execute(build_add_joint_command(&state.document.to_scene(), joint));
}

/// Add a box primitive.
pub fn add_box(state: &mut crate::app::AppState) {
    let cmd = kpe_parametric::add_box_command(&state.document.to_scene());
    state.execute(cmd);
}

/// Add a cylinder primitive.
pub fn add_cylinder(state: &mut crate::app::AppState) {
    let cmd = kpe_parametric::add_cylinder_command(&state.document.to_scene());
    state.execute(cmd);
}

/// Add a sphere primitive.
pub fn add_sphere(state: &mut crate::app::AppState) {
    let cmd = kpe_parametric::add_sphere_command(&state.document.to_scene());
    state.execute(cmd);
}

/// Add a sketch primitive.
pub fn add_sketch(state: &mut crate::app::AppState) {
    let cmd = kpe_parametric::add_sketch_command(&state.document.to_scene());
    state.execute(cmd);
}
