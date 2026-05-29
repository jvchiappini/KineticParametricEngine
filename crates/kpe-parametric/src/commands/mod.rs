pub mod features;

use kpe_schema::geometry::{
    BoxDef, CylinderDef, GeometryNode, GeometryNodeType, SketchDef, SphereDef,
};
use kpe_schema::joint::Joint;

use crate::scene::GeometryScene;

// ── Command trait ─────────────────────────────────────────────────

/// A single undoable parametric operation.
pub trait Command: Send + Sync {
    /// Apply the command to the scene.
    fn execute(&mut self, scene: &mut GeometryScene);
    /// Reverse the command.
    fn undo(&mut self, scene: &mut GeometryScene);
    /// Human-readable label for undo/redo UI.
    fn description(&self) -> &str;
}

// ── Tree traversal utilities ──────────────────────────────────────

/// Find a node by ID in the scene tree.
pub fn find_node<'a>(node: &'a GeometryNode, target_id: &str) -> Option<&'a GeometryNode> {
    if node.id == target_id {
        return Some(node);
    }
    for child in &node.children {
        if let result @ Some(_) = find_node(child, target_id) {
            return result;
        }
    }
    None
}

/// Find the parent of a node by the child's ID.
pub fn find_parent<'a>(node: &'a GeometryNode, target: &str) -> Option<&'a GeometryNode> {
    for child in &node.children {
        if child.id == target {
            return Some(node);
        }
        if let found @ Some(_) = find_parent(child, target) {
            return found;
        }
    }
    None
}

/// Mutable version of `find_parent`.
pub fn find_parent_mut<'a>(
    node: &'a mut GeometryNode,
    target: &str,
) -> Option<&'a mut GeometryNode> {
    for child in &mut node.children {
        if child.id == target {
            return Some(node);
        }
    }
    for child in &mut node.children {
        if let found @ Some(_) = find_parent_mut(child, target) {
            return found;
        }
    }
    None
}

/// Collect all node IDs from the tree.
pub fn collect_ids(node: &GeometryNode, ids: &mut Vec<String>) {
    ids.push(node.id.clone());
    for child in &node.children {
        collect_ids(child, ids);
    }
}

/// Add a child node to the target parent.
pub fn add_child(node: &mut GeometryNode, target: &str, new_child: &GeometryNode) {
    if node.id == target {
        node.children.push(new_child.clone());
        return;
    }
    for child in &mut node.children {
        add_child(child, target, new_child);
    }
}

/// Remove a child node by ID from the tree.
pub fn remove_child(node: &mut GeometryNode, target: &str) {
    node.children.retain(|c| c.id != target);
    for child in &mut node.children {
        remove_child(child, target);
    }
}

fn collect_all_ids(node: &GeometryNode, ids: &mut Vec<String>) {
    ids.push(node.id.clone());
    for child in &node.children {
        collect_all_ids(child, ids);
    }
}

fn find_node_mut<'a>(node: &'a mut GeometryNode, target: &str) -> Option<&'a mut GeometryNode> {
    if node.id == target {
        return Some(node);
    }
    for child in &mut node.children {
        if let found @ Some(_) = find_node_mut(child, target) {
            return found;
        }
    }
    None
}

fn generate_new_id(node: &GeometryNode, existing_ids: &[String]) -> String {
    let base = node.id.trim_end_matches(|c: char| c.is_ascii_digit());
    let mut counter = 0;
    loop {
        counter += 1;
        let candidate = format!("{}{:03}", base, counter);
        if !existing_ids.contains(&candidate) {
            return candidate;
        }
    }
}

/// Reassign IDs for a node and its children to avoid conflicts.
pub fn reassign_ids(node: &mut GeometryNode, existing_ids: &mut Vec<String>) {
    let new_id = generate_new_id(node, existing_ids);
    existing_ids.push(new_id.clone());
    node.id = new_id;
    for child in &mut node.children {
        reassign_ids(child, existing_ids);
    }
}

/// Compute the next available counter value for a given ID prefix.
pub fn next_counter(scene: &GeometryNode, prefix: &str) -> usize {
    let mut ids = Vec::new();
    collect_all_ids(scene, &mut ids);
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

// ── SetParameterCommand ──────────────────────────────────────────

/// Command to change a single numeric parameter on a node.
#[derive(Clone)]
pub struct SetParameterCommand {
    pub node_id: String,
    pub param_name: String,
    pub old_value: f64,
    pub new_value: f64,
}

impl Command for SetParameterCommand {
    fn execute(&mut self, scene: &mut GeometryScene) {
        apply_param(&mut scene.scene, &self.node_id, &self.param_name, self.new_value);
    }

    fn undo(&mut self, scene: &mut GeometryScene) {
        apply_param(&mut scene.scene, &self.node_id, &self.param_name, self.old_value);
    }

    fn description(&self) -> &str {
        "Set Parameter"
    }
}

fn apply_param(node: &mut GeometryNode, target: &str, name: &str, value: f64) {
    if node.id != target {
        for child in &mut node.children {
            apply_param(child, target, name, value);
        }
        return;
    }
    match &mut node.node_type {
        GeometryNodeType::Box(b) => match name {
            "width" => b.width = value,
            "height" => b.height = value,
            "depth" => b.depth = value,
            _ => {}
        },
        GeometryNodeType::Cylinder(c) => match name {
            "radius" => c.radius = value,
            "height" => c.height = value,
            _ => {}
        },
        GeometryNodeType::Sphere(s) => {
            if name == "radius" {
                s.radius = value;
            }
        }
        GeometryNodeType::Extrude(e) => match name {
            "distance" => e.distance = value,
            "taper_angle" => e.taper_angle = if value == 0.0 { None } else { Some(value) },
            _ => {}
        },
        GeometryNodeType::Revolve(r) => {
            if name == "angle" {
                r.angle = value;
            }
        }
        _ => {}
    }
}

// ── AddFeatureCommand ────────────────────────────────────────────

/// Command to add a new child node to a parent.
pub struct AddFeatureCommand {
    pub parent_id: String,
    pub node: GeometryNode,
}

impl Command for AddFeatureCommand {
    fn execute(&mut self, scene: &mut GeometryScene) {
        add_child(&mut scene.scene, &self.parent_id, &self.node);
    }

    fn undo(&mut self, scene: &mut GeometryScene) {
        remove_child(&mut scene.scene, &self.node.id);
    }

    fn description(&self) -> &str {
        "Add Feature"
    }
}

// ── DeleteFeatureCommand ─────────────────────────────────────────

/// Command to delete a feature (node) from the scene tree.
pub struct DeleteFeatureCommand {
    pub parent_id: String,
    pub node: GeometryNode,
}

impl Command for DeleteFeatureCommand {
    fn execute(&mut self, scene: &mut GeometryScene) {
        remove_child(&mut scene.scene, &self.node.id);
    }

    fn undo(&mut self, scene: &mut GeometryScene) {
        add_child(&mut scene.scene, &self.parent_id, &self.node);
    }

    fn description(&self) -> &str {
        "Delete Feature"
    }
}

// ── SetSketchCommand ─────────────────────────────────────────────

/// Command to replace the sketch definition on a node.
pub struct SetSketchCommand {
    pub node_id: String,
    pub old_sketch: Option<SketchDef>,
    pub new_sketch: SketchDef,
}

impl SetSketchCommand {
    pub fn new(node_id: String, new_sketch: SketchDef, scene: &GeometryScene) -> Self {
        let current = get_sketch_def(&scene.scene, &node_id);
        Self {
            node_id,
            old_sketch: current,
            new_sketch,
        }
    }
}

impl Command for SetSketchCommand {
    fn execute(&mut self, scene: &mut GeometryScene) {
        set_sketch_def(&mut scene.scene, &self.node_id, &self.new_sketch);
    }

    fn undo(&mut self, scene: &mut GeometryScene) {
        if let Some(ref old) = self.old_sketch {
            set_sketch_def(&mut scene.scene, &self.node_id, old);
        }
    }

    fn description(&self) -> &str {
        "Edit Sketch"
    }
}

fn get_sketch_def(node: &GeometryNode, target: &str) -> Option<SketchDef> {
    if node.id == target {
        if let GeometryNodeType::Sketch(ref def) = node.node_type {
            return Some(def.clone());
        }
        return None;
    }
    for child in &node.children {
        if let result @ Some(_) = get_sketch_def(child, target) {
            return result;
        }
    }
    None
}

fn set_sketch_def(node: &mut GeometryNode, target: &str, def: &SketchDef) {
    if node.id == target {
        if let GeometryNodeType::Sketch(ref mut existing) = node.node_type {
            *existing = def.clone();
        }
        return;
    }
    for child in &mut node.children {
        set_sketch_def(child, target, def);
    }
}

// ── Joint commands ──────────────────────────────────────────────

/// Command to add a new joint.
pub struct AddJointCommand {
    pub joint: Joint,
}

impl Command for AddJointCommand {
    fn execute(&mut self, scene: &mut GeometryScene) {
        scene.joints.push(self.joint.clone());
    }

    fn undo(&mut self, scene: &mut GeometryScene) {
        scene.joints.retain(|j| j.id != self.joint.id);
    }

    fn description(&self) -> &str {
        "Add Joint"
    }
}

/// Command to change a joint's current value.
pub struct SetJointValueCommand {
    pub joint_id: String,
    pub old_value: f64,
    pub new_value: f64,
}

impl Command for SetJointValueCommand {
    fn execute(&mut self, scene: &mut GeometryScene) {
        if let Some(j) = scene.joints.iter_mut().find(|j| j.id == self.joint_id) {
            j.current_value = self.new_value;
        }
    }

    fn undo(&mut self, scene: &mut GeometryScene) {
        if let Some(j) = scene.joints.iter_mut().find(|j| j.id == self.joint_id) {
            j.current_value = self.old_value;
        }
    }

    fn description(&self) -> &str {
        "Set Joint Value"
    }
}

// ── CompoundCommand ──────────────────────────────────────────────

/// A command composed of multiple sub-commands executed in order.
pub struct CompoundCommand {
    pub commands: Vec<Box<dyn Command>>,
    pub label: String,
}

impl Command for CompoundCommand {
    fn execute(&mut self, scene: &mut GeometryScene) {
        for cmd in &mut self.commands {
            cmd.execute(scene);
        }
    }

    fn undo(&mut self, scene: &mut GeometryScene) {
        for cmd in self.commands.iter_mut().rev() {
            cmd.undo(scene);
        }
    }

    fn description(&self) -> &str {
        &self.label
    }
}

// ── Basic primitive adders ──────────────────────────────────────

/// Create a command that adds a box primitive.
pub fn add_box_command(scene: &GeometryScene) -> Box<dyn Command> {
    let target_id = determine_target(scene);
    let new_id = format!("Box_{:03}", next_counter(&scene.scene, "Box_"));
    let node = GeometryNode {
        id: new_id,
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
    Box::new(AddFeatureCommand {
        parent_id: target_id,
        node,
    })
}

/// Create a command that adds a cylinder primitive.
pub fn add_cylinder_command(scene: &GeometryScene) -> Box<dyn Command> {
    let target_id = determine_target(scene);
    let new_id = format!("Cylinder_{:03}", next_counter(&scene.scene, "Cylinder_"));
    let node = GeometryNode {
        id: new_id,
        node_type: GeometryNodeType::Cylinder(CylinderDef {
            radius: 1.0,
            height: 3.0,
        }),
        transform: None,
        children: vec![],
        operations: vec![],
        color: None,
    };
    Box::new(AddFeatureCommand {
        parent_id: target_id,
        node,
    })
}

/// Create a command that adds a sphere primitive.
pub fn add_sphere_command(scene: &GeometryScene) -> Box<dyn Command> {
    let target_id = determine_target(scene);
    let new_id = format!("Sphere_{:03}", next_counter(&scene.scene, "Sphere_"));
    let node = GeometryNode {
        id: new_id,
        node_type: GeometryNodeType::Sphere(SphereDef { radius: 1.5 }),
        transform: None,
        children: vec![],
        operations: vec![],
        color: None,
    };
    Box::new(AddFeatureCommand {
        parent_id: target_id,
        node,
    })
}

/// Create a command that adds a sketch primitive.
pub fn add_sketch_command(scene: &GeometryScene) -> Box<dyn Command> {
    let target_id = determine_target(scene);
    let new_id = format!("Sketch_{:03}", next_counter(&scene.scene, "Sketch_"));
    let node = GeometryNode {
        id: new_id,
        node_type: GeometryNodeType::Sketch(SketchDef {
            primitives: vec![kpe_schema::geometry::SketchPrimitive::Rectangle {
                x: -2.0,
                y: -2.0,
                width: 4.0,
                height: 4.0,
            }],
            plane: kpe_schema::geometry::SketchPlane::XY,
            extrude: None,
        }),
        transform: None,
        children: vec![],
        operations: vec![],
        color: None,
    };
    Box::new(AddFeatureCommand {
        parent_id: target_id,
        node,
    })
}

fn determine_target(_scene: &GeometryScene) -> String {
    "Root".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_parameter_execute_undo() {
        let mut scene = GeometryScene::new();
        let mut cmd = SetParameterCommand {
            node_id: "Root".to_string(),
            param_name: "width".to_string(),
            old_value: 2.0,
            new_value: 5.0,
        };
        // Not a Box, so width won't apply. This tests that the command
        // runs without error — the param only applies to Box nodes.
        cmd.execute(&mut scene);
        cmd.undo(&mut scene);
    }

    #[test]
    fn test_add_feature_then_delete() {
        let mut scene = GeometryScene::new();
        let node = GeometryNode {
            id: "Box_001".into(),
            node_type: GeometryNodeType::Box(BoxDef {
                width: 1.0,
                height: 1.0,
                depth: 1.0,
            }),
            transform: None,
            children: vec![],
            operations: vec![],
            color: None,
        };
        let mut add_cmd = AddFeatureCommand {
            parent_id: "Root".into(),
            node: node.clone(),
        };
        add_cmd.execute(&mut scene);
        assert!(find_node(&scene.scene, "Box_001").is_some());

        let mut del_cmd = DeleteFeatureCommand {
            parent_id: "Root".into(),
            node,
        };
        del_cmd.execute(&mut scene);
        assert!(find_node(&scene.scene, "Box_001").is_none());

        del_cmd.undo(&mut scene);
        assert!(find_node(&scene.scene, "Box_001").is_some());
    }

    #[test]
    fn test_reassign_ids() {
        let mut node = GeometryNode {
            id: "Box".into(),
            node_type: GeometryNodeType::Compound,
            transform: None,
            children: vec![],
            operations: vec![],
            color: None,
        };
        let mut ids = vec!["Box".to_string()];
        reassign_ids(&mut node, &mut ids);
        assert_ne!(node.id, "Box");
        assert_eq!(node.id, "Box001");
    }
}
