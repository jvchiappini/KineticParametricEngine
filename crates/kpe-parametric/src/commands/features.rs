use kpe_schema::geometry::{
    AssemblyDef, ChamferDef, FilletDef, GeometryNode, GeometryNodeType, TransformOp,
};
use kpe_schema::joint::Joint;
use crate::scene::GeometryScene;

use super::{
    find_node, find_parent, collect_ids, reassign_ids,
    AddFeatureCommand, Command, DeleteFeatureCommand, CompoundCommand,
};

/// Parameters for creating an array of copies.
#[derive(Debug, Clone)]
pub struct ArrayParams {
    pub count: usize,
    pub dx: f64,
    pub dy: f64,
    pub dz: f64,
    pub rx: f64,
    pub ry: f64,
    pub rz: f64,
    pub sx: f64,
    pub sy: f64,
    pub sz: f64,
}

impl Default for ArrayParams {
    fn default() -> Self {
        Self {
            count: 3,
            dx: 4.0,
            dy: 0.0,
            dz: 0.0,
            rx: 0.0,
            ry: 0.0,
            rz: 0.0,
            sx: 1.0,
            sy: 1.0,
            sz: 1.0,
        }
    }
}

// ── Duplicate ────────────────────────────────────────────────────

/// Build a command that duplicates the given node.
pub fn build_duplicate_command(
    scene: &GeometryScene,
    node_id: &str,
) -> Option<Box<dyn Command>> {
    let node = find_node(&scene.scene, node_id)?.clone();
    let parent_id = find_parent(&scene.scene, node_id)
        .map_or("Root".to_string(), |p| p.id.clone());

    let mut existing_ids = Vec::new();
    collect_ids(&scene.scene, &mut existing_ids);

    let mut dup = node;
    reassign_ids(&mut dup, &mut existing_ids);

    Some(Box::new(AddFeatureCommand {
        parent_id,
        node: dup,
    }))
}

// ── Array ────────────────────────────────────────────────────────

/// Build a compound command that creates an array of copies.
pub fn build_array_command(
    scene: &GeometryScene,
    node_id: &str,
    params: &ArrayParams,
) -> Option<Box<dyn Command>> {
    let node = find_node(&scene.scene, node_id)?.clone();
    let parent_id = find_parent(&scene.scene, node_id)
        .map_or("Root".to_string(), |p| p.id.clone());

    let mut existing_ids = Vec::new();
    collect_ids(&scene.scene, &mut existing_ids);

    let mut commands: Vec<Box<dyn Command>> = Vec::new();

    for i in 1..params.count {
        let mut dup = node.clone();
        reassign_ids(&mut dup, &mut existing_ids);
        let t = dup.transform.get_or_insert_with(|| TransformOp {
            translation: None,
            rotation: None,
            scale: None,
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
        commands.push(Box::new(AddFeatureCommand {
            parent_id: parent_id.clone(),
            node: dup,
        }));
    }

    Some(Box::new(CompoundCommand {
        commands,
        label: "Array".to_string(),
    }))
}

// ── Mirror ──────────────────────────────────────────────────────

/// Mirror plane specification.
pub enum MirrorPlane {
    XY,
    XZ,
    YZ,
}

impl MirrorPlane {
    fn scale_factors(&self) -> [f64; 3] {
        match self {
            MirrorPlane::XY => [1.0, 1.0, -1.0],
            MirrorPlane::XZ => [1.0, -1.0, 1.0],
            MirrorPlane::YZ => [-1.0, 1.0, 1.0],
        }
    }
}

/// Build a command that mirrors the given node across a plane.
pub fn build_mirror_command(
    scene: &GeometryScene,
    node_id: &str,
    plane: &MirrorPlane,
) -> Option<Box<dyn Command>> {
    let node = find_node(&scene.scene, node_id)?.clone();
    let parent_id = find_parent(&scene.scene, node_id)
        .map_or("Root".to_string(), |p| p.id.clone());

    let mut existing_ids = Vec::new();
    collect_ids(&scene.scene, &mut existing_ids);

    let mut dup = node;
    reassign_ids(&mut dup, &mut existing_ids);

    let t = dup.transform.get_or_insert_with(|| TransformOp {
        translation: None,
        rotation: None,
        scale: None,
    });
    t.scale = Some(plane.scale_factors());

    Some(Box::new(AddFeatureCommand {
        parent_id,
        node: dup,
    }))
}

// ── Fillet ──────────────────────────────────────────────────────

/// Build a command that wraps the target node in a fillet operation.
pub fn build_fillet_command(
    scene: &GeometryScene,
    node_id: &str,
    radius: f64,
) -> Option<Box<dyn Command>> {
    let node = find_node(&scene.scene, node_id)?.clone();
    let parent_id = find_parent(&scene.scene, node_id)
        .map_or("Root".to_string(), |p| p.id.clone());

    let mut existing_ids = Vec::new();
    collect_ids(&scene.scene, &mut existing_ids);

    let mut fillet_node = GeometryNode {
        id: format!("Fillet_{}", node.id),
        node_type: GeometryNodeType::Fillet(FilletDef { radius }),
        transform: None,
        children: vec![node.clone()],
        operations: vec![],
        color: None,
    };
    reassign_ids(&mut fillet_node, &mut existing_ids);

    // Two operations: delete original, add fillet wrapper
    let del_cmd = Box::new(DeleteFeatureCommand {
        parent_id: parent_id.clone(),
        node: node.clone(),
    }) as Box<dyn Command>;

    let add_cmd = Box::new(AddFeatureCommand {
        parent_id,
        node: fillet_node,
    }) as Box<dyn Command>;

    Some(Box::new(CompoundCommand {
        commands: vec![del_cmd, add_cmd],
        label: "Add Fillet".to_string(),
    }))
}

// ── Chamfer ─────────────────────────────────────────────────────

/// Build a command that wraps the target node in a chamfer operation.
pub fn build_chamfer_command(
    scene: &GeometryScene,
    node_id: &str,
    distance: f64,
) -> Option<Box<dyn Command>> {
    let node = find_node(&scene.scene, node_id)?.clone();
    let parent_id = find_parent(&scene.scene, node_id)
        .map_or("Root".to_string(), |p| p.id.clone());

    let mut existing_ids = Vec::new();
    collect_ids(&scene.scene, &mut existing_ids);

    let mut chamfer_node = GeometryNode {
        id: format!("Chamfer_{}", node.id),
        node_type: GeometryNodeType::Chamfer(ChamferDef { distance }),
        transform: None,
        children: vec![node.clone()],
        operations: vec![],
        color: None,
    };
    reassign_ids(&mut chamfer_node, &mut existing_ids);

    let del_cmd = Box::new(DeleteFeatureCommand {
        parent_id: parent_id.clone(),
        node: node.clone(),
    }) as Box<dyn Command>;

    let add_cmd = Box::new(AddFeatureCommand {
        parent_id,
        node: chamfer_node,
    }) as Box<dyn Command>;

    Some(Box::new(CompoundCommand {
        commands: vec![del_cmd, add_cmd],
        label: "Add Chamfer".to_string(),
    }))
}

// ── Group ───────────────────────────────────────────────────────

/// Build a command that wraps the target node in a Compound group.
pub fn build_group_command(
    scene: &GeometryScene,
    node_id: &str,
) -> Option<Box<dyn Command>> {
    let node = find_node(&scene.scene, node_id)?.clone();
    let parent_id = find_parent(&scene.scene, node_id)
        .map_or("Root".to_string(), |p| p.id.clone());

    let group = GeometryNode {
        id: format!("Group_{}", node.id),
        node_type: GeometryNodeType::Compound,
        transform: None,
        children: vec![node.clone()],
        operations: vec![],
        color: None,
    };

    let del_cmd = Box::new(DeleteFeatureCommand {
        parent_id: parent_id.clone(),
        node: node.clone(),
    }) as Box<dyn Command>;

    let add_cmd = Box::new(AddFeatureCommand {
        parent_id,
        node: group,
    }) as Box<dyn Command>;

    Some(Box::new(CompoundCommand {
        commands: vec![del_cmd, add_cmd],
        label: "Group".to_string(),
    }))
}

// ── Assembly ────────────────────────────────────────────────────

/// Build a command that wraps the target node in an Assembly.
pub fn build_assembly_command(
    scene: &GeometryScene,
    node_id: &str,
) -> Option<Box<dyn Command>> {
    let node = find_node(&scene.scene, node_id)?.clone();
    let parent_id = find_parent(&scene.scene, node_id)
        .map_or("Root".to_string(), |p| p.id.clone());

    let assembly = GeometryNode {
        id: format!("Assembly_{}", node.id),
        node_type: GeometryNodeType::Assembly(AssemblyDef { name: None }),
        transform: None,
        children: vec![node.clone()],
        operations: vec![],
        color: None,
    };

    let del_cmd = Box::new(DeleteFeatureCommand {
        parent_id: parent_id.clone(),
        node: node.clone(),
    }) as Box<dyn Command>;

    let add_cmd = Box::new(AddFeatureCommand {
        parent_id,
        node: assembly,
    }) as Box<dyn Command>;

    Some(Box::new(CompoundCommand {
        commands: vec![del_cmd, add_cmd],
        label: "Assembly".to_string(),
    }))
}

// ── Delete ──────────────────────────────────────────────────────

/// Build a command that deletes a node from the scene.
pub fn build_delete_command(
    scene: &GeometryScene,
    node_id: &str,
) -> Option<Box<dyn Command>> {
    let node = find_node(&scene.scene, node_id)?.clone();
    let parent_id = find_parent(&scene.scene, node_id)
        .map_or("Root".to_string(), |p| p.id.clone());

    Some(Box::new(DeleteFeatureCommand {
        parent_id,
        node,
    }))
}

/// Build a compound command that deletes multiple nodes.
pub fn build_delete_multi_command(
    scene: &GeometryScene,
    node_ids: &[String],
) -> Option<Box<dyn Command>> {
    let mut commands: Vec<Box<dyn Command>> = Vec::new();
    for id in node_ids {
        if let Some(node) = find_node(&scene.scene, id) {
            let parent_id = find_parent(&scene.scene, id)
                .map_or("Root".to_string(), |p| p.id.clone());
            commands.push(Box::new(DeleteFeatureCommand {
                parent_id,
                node: node.clone(),
            }) as Box<dyn Command>);
        }
    }
    if commands.is_empty() {
        None
    } else {
        Some(Box::new(CompoundCommand {
            commands,
            label: "Delete Selected".to_string(),
        }))
    }
}

// ── Joint ───────────────────────────────────────────────────────

/// Build a command that creates a joint between a parent and child.
pub fn build_add_joint_command(
    _scene: &GeometryScene,
    joint: Joint,
) -> Box<dyn Command> {
    Box::new(super::AddJointCommand { joint })
}

#[cfg(test)]
mod tests {
    use super::*;
    use kpe_schema::geometry::{BoxDef, GeometryNode};
    use crate::commands::add_child;

    fn make_scene_with_box(box_id: &str) -> GeometryScene {
        let mut scene = GeometryScene::new();
        let node = GeometryNode {
            id: box_id.into(),
            node_type: GeometryNodeType::Box(BoxDef {
                width: 2.0,
                height: 2.0,
                depth: 2.0,
            }),
            transform: None,
            children: vec![],
            operations: vec![],
            color: None,
        };
        add_child(&mut scene.scene, "Root", &node);
        scene
    }

    #[test]
    fn test_build_duplicate_command() {
        let scene = make_scene_with_box("Box_001");
        let cmd = build_duplicate_command(&scene, "Box_001");
        assert!(cmd.is_some());
    }

    #[test]
    fn test_build_fillet_command() {
        let scene = make_scene_with_box("Box_001");
        let cmd = build_fillet_command(&scene, "Box_001", 0.5);
        assert!(cmd.is_some());
    }

    #[test]
    fn test_build_mirror_command() {
        let scene = make_scene_with_box("Box_001");
        let cmd = build_mirror_command(&scene, "Box_001", &MirrorPlane::XY);
        assert!(cmd.is_some());
    }
}
