use std::collections::HashMap;
use kpe_geometry::mesh::build_mesh_from_node;
use kpe_schema::geometry::{GeometryNode, GeometryNodeType, TriangleMesh};
use kpe_schema::recipe::KPERecipe;

#[derive(Debug, Clone)]
pub struct SceneGeometry {
    pub meshes: HashMap<String, TriangleMesh>,
}

impl SceneGeometry {
    pub fn new() -> Self {
        Self { meshes: HashMap::new() }
    }

    pub fn triangle_count(&self) -> usize {
        self.meshes.values().map(|m| m.triangles.len()).sum()
    }
}

impl Default for SceneGeometry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Document {
    pub recipe: KPERecipe,
    pub evaluated: SceneGeometry,
    pub selection: Option<String>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            recipe: KPERecipe::default(),
            evaluated: SceneGeometry::new(),
            selection: None,
        }
    }

    pub fn evaluate_all(&mut self) {
        let node = &self.recipe.scene;
        let mut meshes = HashMap::new();
        collect_meshes(node, &mut meshes);
        self.evaluated = SceneGeometry { meshes };
    }

    pub fn evaluate_node(&mut self, node_id: &str) {
        if let Some(mesh) = find_node_mesh(&self.recipe.scene, node_id) {
            self.evaluated.meshes.insert(node_id.to_string(), mesh);
        }
    }

    pub fn all_node_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();
        collect_ids(&self.recipe.scene, &mut ids);
        ids
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

fn collect_meshes(node: &GeometryNode, meshes: &mut HashMap<String, TriangleMesh>) {
    if !matches!(node.node_type, GeometryNodeType::Compound) {
        let mesh = build_mesh_from_node(node);
        meshes.insert(node.id.clone(), mesh);
    }
    for child in &node.children {
        collect_meshes(child, meshes);
    }
}

fn find_node_mesh(node: &GeometryNode, target_id: &str) -> Option<TriangleMesh> {
    if node.id == target_id {
        return Some(build_mesh_from_node(node));
    }
    for child in &node.children {
        if let result @ Some(_) = find_node_mesh(child, target_id) {
            return result;
        }
    }
    None
}

fn collect_ids(node: &GeometryNode, ids: &mut Vec<String>) {
    ids.push(node.id.clone());
    for child in &node.children {
        collect_ids(child, ids);
    }
}
