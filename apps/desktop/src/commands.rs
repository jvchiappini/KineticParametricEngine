use crate::document::Document;
use kpe_schema::geometry::{GeometryNode, GeometryNodeType};
use kpe_schema::joint::Joint;

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
        GeometryNodeType::Extrude(e) => {
            match name {
                "distance" => e.distance = value,
                "taper_angle" => e.taper_angle = if value == 0.0 { None } else { Some(value) },
                _ => {}
            }
        }
        GeometryNodeType::Revolve(r) => {
            if name == "angle" {
                r.angle = value;
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

pub struct SetSketchCommand {
    pub node_id: String,
    pub old_sketch: Option<kpe_schema::geometry::SketchDef>,
    pub new_sketch: kpe_schema::geometry::SketchDef,
}

impl Command for SetSketchCommand {
    fn execute(&mut self, doc: &mut Document) {
        // Capture old sketch on first execute
        if self.old_sketch.is_none() {
            let current = get_sketch_def(&doc.recipe.scene, &self.node_id);
            self.old_sketch = current;
        }
        set_sketch_def(&mut doc.recipe.scene, &self.node_id, &self.new_sketch);
        doc.evaluate_all();
    }

    fn undo(&mut self, doc: &mut Document) {
        if let Some(ref old) = self.old_sketch {
            set_sketch_def(&mut doc.recipe.scene, &self.node_id, old);
            doc.evaluate_all();
        }
    }

    fn description(&self) -> &str {
        "Edit Sketch"
    }
}

fn get_sketch_def(node: &GeometryNode, target: &str) -> Option<kpe_schema::geometry::SketchDef> {
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

fn set_sketch_def(node: &mut GeometryNode, target: &str, def: &kpe_schema::geometry::SketchDef) {
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

pub fn add_child(node: &mut GeometryNode, target: &str, new_child: &GeometryNode) {
    if node.id == target {
        node.children.push(new_child.clone());
        return;
    }
    for child in &mut node.children {
        add_child(child, target, new_child);
    }
}

pub fn remove_child(node: &mut GeometryNode, target: &str) {
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

pub fn find_parent_mut<'a>(node: &'a mut GeometryNode, target: &str) -> Option<&'a mut GeometryNode> {
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

pub fn collect_ids(node: &GeometryNode, ids: &mut Vec<String>) {
    ids.push(node.id.clone());
    for child in &node.children {
        collect_ids(child, ids);
    }
}

pub fn reassign_ids(node: &mut GeometryNode, existing_ids: &mut Vec<String>) {
    let new_id = generate_new_id(node, existing_ids);
    existing_ids.push(new_id.clone());
    node.id = new_id;
    for child in &mut node.children {
        reassign_ids(child, existing_ids);
    }
}

pub fn copy_selected(state: &mut crate::app::AppState) {
    let selected = match &state.document.selection {
        Some(id) => id.clone(),
        None => return,
    };
    let node = find_node(&state.document.recipe.scene, &selected);
    if let Some(n) = node {
        state.clipboard = Some(n.clone());
    }
}

pub fn find_node<'a>(node: &'a GeometryNode, target: &str) -> Option<&'a GeometryNode> {
    if node.id == target { return Some(node); }
    for child in &node.children {
        if let found @ Some(_) = find_node(child, target) {
            return found;
        }
    }
    None
}

pub fn cut_selected(state: &mut crate::app::AppState) {
    let selected = match &state.document.selection {
        Some(id) => id.clone(),
        None => return,
    };
    let node = find_node(&state.document.recipe.scene, &selected);
    let cloned = node.cloned();
    if cloned.is_none() { return; }
    let cloned = cloned.unwrap();
    let parent = find_parent(&state.document.recipe.scene, &selected);
    let parent_id = parent.map(|p| p.id.clone());

    let mut cmd = DeleteFeatureCommand {
        parent_id: parent_id.unwrap_or_default(),
        node: cloned.clone(),
    };
    cmd.execute(&mut state.document);
    state.history.undo_stack.push(Box::new(cmd));
    state.clipboard = Some(cloned);
    state.document.selection = None;
    state.mark_dirty();
}

#[derive(Clone)]
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
        Self { count: 3, dx: 4.0, dy: 0.0, dz: 0.0, rx: 0.0, ry: 0.0, rz: 0.0, sx: 1.0, sy: 1.0, sz: 1.0 }
    }
}

pub fn paste_clipboard(state: &mut crate::app::AppState) {
    let clip: kpe_schema::geometry::GeometryNode = match &state.clipboard {
        Some(c) => c.clone(),
        None => return,
    };

    let selected = state.document.selection.clone();
    let target_id = if let Some(ref id) = selected {
        let is_valid = find_node(&state.document.recipe.scene, id).is_some();
        if is_valid {
            find_parent(&state.document.recipe.scene, id)
                .map_or("Root".to_string(), |p| p.id.clone())
        } else {
            "Root".to_string()
        }
    } else {
        "Root".to_string()
    };

    let mut existing_ids = Vec::new();
    collect_ids(&state.document.recipe.scene, &mut existing_ids);

    let mut pasted = clip;
    reassign_ids(&mut pasted, &mut existing_ids);

    let cmd = AddFeatureCommand {
        parent_id: target_id,
        node: pasted,
    };
    state.history.execute(Box::new(cmd), &mut state.document);
    state.mark_dirty();
}

pub struct AddJointCommand {
    pub joint: Joint,
}

impl Command for AddJointCommand {
    fn execute(&mut self, doc: &mut Document) {
        doc.recipe.joints.push(self.joint.clone());
        doc.evaluate_all();
    }

    fn undo(&mut self, doc: &mut Document) {
        doc.recipe.joints.retain(|j| j.id != self.joint.id);
        doc.evaluate_all();
    }

    fn description(&self) -> &str {
        "Add Joint"
    }
}

pub struct SetJointValueCommand {
    pub joint_id: String,
    pub old_value: f64,
    pub new_value: f64,
}

impl Command for SetJointValueCommand {
    fn execute(&mut self, doc: &mut Document) {
        if let Some(j) = doc.recipe.joints.iter_mut().find(|j| j.id == self.joint_id) {
            j.current_value = self.new_value;
        }
        doc.evaluate_all();
    }

    fn undo(&mut self, doc: &mut Document) {
        if let Some(j) = doc.recipe.joints.iter_mut().find(|j| j.id == self.joint_id) {
            j.current_value = self.old_value;
        }
        doc.evaluate_all();
    }

    fn description(&self) -> &str {
        "Set Joint Value"
    }
}

pub use crate::feature_commands::*;
