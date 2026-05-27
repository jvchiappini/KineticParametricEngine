use crate::document::Document;
use kpe_schema::geometry::{GeometryNode, GeometryNodeType};

pub trait Command: Send + Sync {
    fn execute(&mut self, doc: &mut Document);
    fn undo(&mut self, doc: &mut Document);
    fn description(&self) -> &str;
}

#[derive(Clone)]
pub struct SetParameterCommand {
    pub node_id: String,
    pub param_name: String,
    pub old_value: f64,
    pub new_value: f64,
}

impl Command for SetParameterCommand {
    fn execute(&mut self, doc: &mut Document) {
        apply_param(&mut doc.recipe.scene, &self.node_id, &self.param_name, self.new_value);
        doc.evaluate_node(&self.node_id);
    }

    fn undo(&mut self, doc: &mut Document) {
        apply_param(&mut doc.recipe.scene, &self.node_id, &self.param_name, self.old_value);
        doc.evaluate_node(&self.node_id);
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
        GeometryNodeType::Box(b) => {
            match name {
                "width" => b.width = value,
                "height" => b.height = value,
                "depth" => b.depth = value,
                _ => {}
            }
        }
        GeometryNodeType::Cylinder(c) => {
            match name {
                "radius" => c.radius = value,
                "height" => c.height = value,
                _ => {}
            }
        }
        GeometryNodeType::Sphere(s) => {
            if name == "radius" {
                s.radius = value;
            }
        }
        _ => {}
    }
}

pub struct AddFeatureCommand {
    pub parent_id: String,
    pub node: GeometryNode,
}

impl Command for AddFeatureCommand {
    fn execute(&mut self, doc: &mut Document) {
        add_child(&mut doc.recipe.scene, &self.parent_id, &self.node);
        doc.evaluate_all();
    }

    fn undo(&mut self, doc: &mut Document) {
        remove_child(&mut doc.recipe.scene, &self.node.id);
        doc.evaluate_all();
    }

    fn description(&self) -> &str {
        "Add Feature"
    }
}

fn add_child(node: &mut GeometryNode, target: &str, new_child: &GeometryNode) {
    if node.id == target {
        node.children.push(new_child.clone());
        return;
    }
    for child in &mut node.children {
        add_child(child, target, new_child);
    }
}

fn remove_child(node: &mut GeometryNode, target: &str) {
    node.children.retain(|c| c.id != target);
    for child in &mut node.children {
        remove_child(child, target);
    }
}

pub struct DeleteFeatureCommand {
    pub parent_id: String,
    pub node: GeometryNode,
}

impl Command for DeleteFeatureCommand {
    fn execute(&mut self, doc: &mut Document) {
        remove_child(&mut doc.recipe.scene, &self.node.id);
        if doc.selection.as_deref() == Some(&self.node.id) {
            doc.selection = None;
        }
        doc.evaluate_all();
    }

    fn undo(&mut self, doc: &mut Document) {
        add_child(&mut doc.recipe.scene, &self.parent_id, &self.node);
        doc.evaluate_all();
    }

    fn description(&self) -> &str {
        "Delete Feature"
    }
}

pub struct CommandHistory {
    pub undo_stack: Vec<Box<dyn Command>>,
    pub redo_stack: Vec<Box<dyn Command>>,
    pub max_undo: usize,
}

impl CommandHistory {
    pub fn new() -> Self {
        Self { undo_stack: Vec::new(), redo_stack: Vec::new(), max_undo: 50 }
    }

    pub fn execute(&mut self, cmd: Box<dyn Command>, doc: &mut Document) {
        let mut cmd = cmd;
        cmd.execute(doc);
        self.undo_stack.push(cmd);
        self.redo_stack.clear();
        if self.undo_stack.len() > self.max_undo {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self, doc: &mut Document) {
        if let Some(mut cmd) = self.undo_stack.pop() {
            cmd.undo(doc);
            self.redo_stack.push(cmd);
        }
    }

    pub fn redo(&mut self, doc: &mut Document) {
        if let Some(mut cmd) = self.redo_stack.pop() {
            cmd.execute(doc);
            self.undo_stack.push(cmd);
        }
    }

    pub fn can_undo(&self) -> bool { !self.undo_stack.is_empty() }
    pub fn can_redo(&self) -> bool { !self.redo_stack.is_empty() }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}
