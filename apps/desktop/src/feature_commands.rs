use crate::commands::{
    find_node, find_parent, collect_ids, reassign_ids,
    AddFeatureCommand, Command, DeleteFeatureCommand, AddJointCommand,
    ArrayParams,
};
use kpe_schema::geometry::{
    GeometryNode, GeometryNodeType, TransformOp,
    FilletDef, ChamferDef, AssemblyDef, BoxDef, CylinderDef,
    SphereDef, SketchDef, SketchPlane, SketchPrimitive,
};
use kpe_schema::joint::{Joint, JointLimits};

pub fn duplicate_selected(state: &mut crate::app::AppState) {
    let selected = match &state.document.selection {
        Some(id) => id.clone(),
        None => return,
    };
    let node = find_node(&state.document.recipe.scene, &selected).cloned();
    let node = match node { Some(n) => n, None => return };
    let parent_id = find_parent(&state.document.recipe.scene, &selected)
        .map_or("Root".to_string(), |p| p.id.clone());
    drop(selected);

    let mut existing_ids = Vec::new();
    collect_ids(&state.document.recipe.scene, &mut existing_ids);

    let mut dup = node.clone();
    reassign_ids(&mut dup, &mut existing_ids);

    let cmd = AddFeatureCommand { parent_id, node: dup };
    state.history.execute(Box::new(cmd), &mut state.document);
    state.mark_dirty();
}

pub fn array_selected(state: &mut crate::app::AppState, params: &ArrayParams) {
    let selected = match &state.document.selection {
        Some(id) => id.clone(),
        None => return,
    };
    let node = find_node(&state.document.recipe.scene, &selected).cloned();
    let node = match node { Some(n) => n, None => return };
    let parent_id = find_parent(&state.document.recipe.scene, &selected)
        .map_or("Root".to_string(), |p| p.id.clone());
    drop(selected);

    let mut existing_ids = Vec::new();
    collect_ids(&state.document.recipe.scene, &mut existing_ids);

    for i in 1..params.count {
        let mut dup = node.clone();
        reassign_ids(&mut dup, &mut existing_ids);
        let t = dup.transform.get_or_insert_with(|| TransformOp {
            translation: None, rotation: None, scale: None,
        });
        let fi = i as f64;
        t.translation = Some([
            params.dx * fi + t.translation.unwrap_or([0.0; 3])[0],
            params.dy * fi + t.translation.unwrap_or([0.0; 3])[1],
            params.dz * fi + t.translation.unwrap_or([0.0; 3])[2],
        ]);
        t.rotation = Some([
            params.rx * fi + t.rotation.unwrap_or([0.0; 3])[0],
            params.ry * fi + t.rotation.unwrap_or([0.0; 3])[1],
            params.rz * fi + t.rotation.unwrap_or([0.0; 3])[2],
        ]);
        t.scale = Some([
            params.sx.powf(fi) * t.scale.unwrap_or([1.0; 3])[0],
            params.sy.powf(fi) * t.scale.unwrap_or([1.0; 3])[1],
            params.sz.powf(fi) * t.scale.unwrap_or([1.0; 3])[2],
        ]);
        let cmd = AddFeatureCommand { parent_id: parent_id.clone(), node: dup };
        state.history.execute(Box::new(cmd), &mut state.document);
    }
    state.mark_dirty();
}

pub fn mirror_selected(state: &mut crate::app::AppState, plane: &str) {
    let selected = match &state.document.selection {
        Some(id) => id.clone(),
        None => return,
    };
    let node = find_node(&state.document.recipe.scene, &selected).cloned();
    let node = match node { Some(n) => n, None => return };
    let parent_id = find_parent(&state.document.recipe.scene, &selected)
        .map_or("Root".to_string(), |p| p.id.clone());
    drop(selected);

    let mut existing_ids = Vec::new();
    collect_ids(&state.document.recipe.scene, &mut existing_ids);

    let mut dup = node.clone();
    reassign_ids(&mut dup, &mut existing_ids);

    let scale = match plane {
        "XY" => [1.0, 1.0, -1.0],
        "XZ" => [1.0, -1.0, 1.0],
        "YZ" => [-1.0, 1.0, 1.0],
        _ => [1.0, 1.0, -1.0],
    };
    let t = dup.transform.get_or_insert_with(|| TransformOp {
        translation: None, rotation: None, scale: None,
    });
    t.scale = Some(scale);

    let cmd = AddFeatureCommand { parent_id, node: dup };
    state.history.execute(Box::new(cmd), &mut state.document);
    state.mark_dirty();
}

pub fn add_fillet(state: &mut crate::app::AppState, radius: f64) {
    let selected = match &state.document.selection {
        Some(id) => id.clone(),
        None => return,
    };
    let node = find_node(&state.document.recipe.scene, &selected).cloned();
    let node = match node { Some(n) => n, None => return };
    let parent_id = find_parent(&state.document.recipe.scene, &selected)
        .map_or("Root".to_string(), |p| p.id.clone());
    drop(selected);

    let mut existing_ids = Vec::new();
    collect_ids(&state.document.recipe.scene, &mut existing_ids);

    let fillet_id = format!("Fillet_{}", node.id);
    let fillet_child = node.clone();

    let mut fillet_node = GeometryNode {
        id: fillet_id.clone(),
        node_type: GeometryNodeType::Fillet(FilletDef { radius }),
        transform: None,
        children: vec![fillet_child],
        operations: vec![],
        color: None,
    };
    reassign_ids(&mut fillet_node, &mut existing_ids);

    let mut cmd = DeleteFeatureCommand {
        parent_id: parent_id.clone(),
        node: node.clone(),
    };
    cmd.execute(&mut state.document);
    state.history.undo_stack.push(Box::new(cmd));
    state.history.redo_stack.clear();

    let add_cmd = AddFeatureCommand { parent_id, node: fillet_node };
    state.history.execute(Box::new(add_cmd), &mut state.document);
    state.mark_dirty();
}

pub fn add_chamfer(state: &mut crate::app::AppState, distance: f64) {
    let selected = match &state.document.selection {
        Some(id) => id.clone(),
        None => return,
    };
    let node = find_node(&state.document.recipe.scene, &selected).cloned();
    let node = match node { Some(n) => n, None => return };
    let parent_id = find_parent(&state.document.recipe.scene, &selected)
        .map_or("Root".to_string(), |p| p.id.clone());
    drop(selected);

    let mut existing_ids = Vec::new();
    collect_ids(&state.document.recipe.scene, &mut existing_ids);

    let mut chamfer_node = GeometryNode {
        id: format!("Chamfer_{}", node.id),
        node_type: GeometryNodeType::Chamfer(ChamferDef { distance }),
        transform: None,
        children: vec![node.clone()],
        operations: vec![],
        color: None,
    };
    reassign_ids(&mut chamfer_node, &mut existing_ids);

    let mut cmd = DeleteFeatureCommand {
        parent_id: parent_id.clone(),
        node: node.clone(),
    };
    cmd.execute(&mut state.document);
    state.history.undo_stack.push(Box::new(cmd));
    state.history.redo_stack.clear();

    let add_cmd = AddFeatureCommand { parent_id, node: chamfer_node };
    state.history.execute(Box::new(add_cmd), &mut state.document);
    state.mark_dirty();
}

pub fn delete_selected_node(state: &mut crate::app::AppState) {
    let selected = match &state.document.selection {
        Some(id) => id.clone(),
        None => return,
    };
    if selected == "Root" { return; }
    let node = find_node(&state.document.recipe.scene, &selected).cloned();
    let node = match node { Some(n) => n, None => return };
    let parent_id = find_parent(&state.document.recipe.scene, &selected)
        .map_or("Root".to_string(), |p| p.id.clone());

    let cmd = DeleteFeatureCommand { parent_id, node };
    state.history.execute(Box::new(cmd), &mut state.document);
    state.mark_dirty();
}

pub fn delete_selected_nodes(state: &mut crate::app::AppState) {
    let mut ids = state.document.multi_selection.clone();
    if let Some(ref sel) = state.document.selection {
        if !ids.contains(sel) && sel != "Root" {
            ids.insert(0, sel.clone());
        }
    }
    if ids.is_empty() { return; }
    for id in &ids {
        let node = find_node(&state.document.recipe.scene, id).cloned();
        let node = match node { Some(n) => n, None => continue };
        let parent_id = find_parent(&state.document.recipe.scene, id)
            .map_or("Root".to_string(), |p| p.id.clone());
        let cmd = DeleteFeatureCommand { parent_id, node };
        state.history.execute(Box::new(cmd), &mut state.document);
    }
    state.document.selection = None;
    state.document.multi_selection.clear();
    state.mark_dirty();
}

pub fn add_group(state: &mut crate::app::AppState, selected_id: &str) {
    let node = find_node(&state.document.recipe.scene, selected_id).cloned();
    let node = match node { Some(n) => n, None => return };
    let parent_id = find_parent(&state.document.recipe.scene, selected_id)
        .map_or("Root".to_string(), |p| p.id.clone());

    let group = GeometryNode {
        id: format!("Group_{}", node.id),
        node_type: GeometryNodeType::Compound,
        transform: None,
        children: vec![node.clone()],
        operations: vec![],
        color: None,
    };

    let mut del_cmd = DeleteFeatureCommand {
        parent_id: parent_id.clone(),
        node: node.clone(),
    };
    del_cmd.execute(&mut state.document);
    state.history.undo_stack.push(Box::new(del_cmd));
    state.history.redo_stack.clear();

    let add_cmd = AddFeatureCommand { parent_id, node: group.clone() };
    state.history.execute(Box::new(add_cmd), &mut state.document);
    state.document.selection = Some(group.id);
    state.mark_dirty();
}

pub fn add_assembly(state: &mut crate::app::AppState, selected_id: &str) {
    let node = find_node(&state.document.recipe.scene, selected_id).cloned();
    let node = match node { Some(n) => n, None => return };
    let parent_id = find_parent(&state.document.recipe.scene, selected_id)
        .map_or("Root".to_string(), |p| p.id.clone());

    let assembly = GeometryNode {
        id: format!("Assembly_{}", node.id),
        node_type: GeometryNodeType::Assembly(AssemblyDef { name: None }),
        transform: None,
        children: vec![node.clone()],
        operations: vec![],
        color: None,
    };

    let mut del_cmd = DeleteFeatureCommand {
        parent_id: parent_id.clone(),
        node: node.clone(),
    };
    del_cmd.execute(&mut state.document);
    state.history.undo_stack.push(Box::new(del_cmd));
    state.history.redo_stack.clear();

    let add_cmd = AddFeatureCommand { parent_id, node: assembly.clone() };
    state.history.execute(Box::new(add_cmd), &mut state.document);
    state.document.selection = Some(assembly.id);
    state.mark_dirty();
}

pub fn add_joint(state: &mut crate::app::AppState, selected_ids: &Option<String>) {
    let child_id = match selected_ids {
        Some(id) => id.clone(),
        None => return,
    };
    let parent = find_parent(&state.document.recipe.scene, &child_id);
    let parent_id = parent.map(|p| p.id.clone()).unwrap_or_else(|| "Root".to_string());

    let joint_id = format!("Joint_{}", state.document.recipe.joints.len() + 1);
    let joint = Joint {
        id: joint_id,
        joint_type: state.new_joint_type.clone(),
        parent_id,
        child_id,
        pivot: state.new_joint_pivot,
        axis: state.new_joint_axis,
        limits: Some(JointLimits { min: -180.0, max: 180.0, damping: None, stiffness: None }),
        current_value: 0.0,
    };

    let cmd = AddJointCommand { joint };
    state.history.execute(Box::new(cmd), &mut state.document);
    state.mark_dirty();
}

pub fn add_box(state: &mut crate::app::AppState) {
    let parent_id = add_target_parent(state);
    let new_id = format!("Box_{:03}", next_counter(state, "Box_"));
    let node = GeometryNode {
        id: new_id,
        node_type: GeometryNodeType::Box(BoxDef { width: 2.0, height: 2.0, depth: 2.0 }),
        transform: None,
        children: vec![],
        operations: vec![],
        color: None,
    };
    let cmd = AddFeatureCommand { parent_id, node };
    state.history.execute(Box::new(cmd), &mut state.document);
    state.mark_dirty();
}

pub fn add_cylinder(state: &mut crate::app::AppState) {
    let parent_id = add_target_parent(state);
    let new_id = format!("Cylinder_{:03}", next_counter(state, "Cylinder_"));
    let node = GeometryNode {
        id: new_id,
        node_type: GeometryNodeType::Cylinder(CylinderDef { radius: 1.0, height: 3.0 }),
        transform: None,
        children: vec![],
        operations: vec![],
        color: None,
    };
    let cmd = AddFeatureCommand { parent_id, node };
    state.history.execute(Box::new(cmd), &mut state.document);
    state.mark_dirty();
}

pub fn add_sphere(state: &mut crate::app::AppState) {
    let parent_id = add_target_parent(state);
    let new_id = format!("Sphere_{:03}", next_counter(state, "Sphere_"));
    let node = GeometryNode {
        id: new_id,
        node_type: GeometryNodeType::Sphere(SphereDef { radius: 1.5 }),
        transform: None,
        children: vec![],
        operations: vec![],
        color: None,
    };
    let cmd = AddFeatureCommand { parent_id, node };
    state.history.execute(Box::new(cmd), &mut state.document);
    state.mark_dirty();
}

pub fn add_sketch(state: &mut crate::app::AppState) {
    let parent_id = add_target_parent(state);
    let new_id = format!("Sketch_{:03}", next_counter(state, "Sketch_"));
    let node = GeometryNode {
        id: new_id,
        node_type: GeometryNodeType::Sketch(SketchDef {
            primitives: vec![SketchPrimitive::Rectangle { x: -2.0, y: -2.0, width: 4.0, height: 4.0 }],
            plane: SketchPlane::XY,
        }),
        transform: None,
        children: vec![],
        operations: vec![],
        color: None,
    };
    let cmd = AddFeatureCommand { parent_id, node };
    state.history.execute(Box::new(cmd), &mut state.document);
    state.mark_dirty();
}

fn add_target_parent(state: &mut crate::app::AppState) -> String {
    if let Some(ref sel) = state.document.selection {
        if find_node(&state.document.recipe.scene, sel).is_some() {
            return sel.clone();
        }
    }
    "Root".to_string()
}

fn next_counter(state: &crate::app::AppState, prefix: &str) -> usize {
    let mut ids = Vec::new();
    collect_ids(&state.document.recipe.scene, &mut ids);
    let mut max = 0usize;
    for id in &ids {
        if let Some(rest) = id.strip_prefix(prefix) {
            if let Ok(n) = rest.parse::<usize>() {
                max = max.max(n);
            }
        }
    }
    max + 1
}
