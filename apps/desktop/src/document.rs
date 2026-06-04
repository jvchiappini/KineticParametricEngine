use std::collections::{HashMap, HashSet};

use kpe_geometry::evaluator::{self, SceneGeometry};
use kpe_parametric::scene::GeometryScene;
use kpe_schema::recipe::KPERecipe;

/// Application document wrapping a recipe with UI state and cached geometry.
#[derive(Debug, Clone)]
pub struct Document {
    pub recipe: KPERecipe,
    pub evaluated: SceneGeometry,
    pub selection: Option<String>,
    pub multi_selection: Vec<String>,
    pub joint_selection: Option<String>,
    /// (node_id, pivot_id) when a pivot item is selected in the tree.
    pub pivot_selection: Option<(String, String)>,
    pub file_path: Option<String>,
    pub is_modified: bool,
    pub hidden_nodes: HashSet<String>,
    node_hashes: HashMap<String, u64>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            recipe: KPERecipe::default(),
            evaluated: SceneGeometry::new(),
            selection: None,
            multi_selection: Vec::new(),
            joint_selection: None,
            pivot_selection: None,
            file_path: None,
            is_modified: false,
            hidden_nodes: HashSet::new(),
            node_hashes: HashMap::new(),
        }
    }

    /// Evaluate all nodes in the recipe scene, using hash caching.
    pub fn evaluate_all(&mut self) {
        let old_meshes = &self.evaluated.meshes;
        let sg = evaluator::evaluate_scene(&self.recipe, &self.node_hashes, old_meshes);
        self.node_hashes = sg.hashes.clone();
        self.evaluated = sg;
    }

    /// Evaluate a single node by ID and update its cached hash.
    ///
    /// For container nodes (Compound/Assembly) this re-evaluates all descendant
    /// leaf nodes so that transform changes propagate correctly to children.
    pub fn evaluate_node(&mut self, node_id: &str) {
        // Check if the node is a container — if so, re-evaluate all descendants.
        let is_container = evaluator::find_node(&self.recipe.scene, node_id)
            .map(|n| matches!(
                n.node_type,
                kpe_schema::geometry::GeometryNodeType::Compound
                    | kpe_schema::geometry::GeometryNodeType::Assembly(_)
            ))
            .unwrap_or(false);

        if is_container {
            // Collect all leaf descendant IDs first to avoid borrow conflicts
            let descendant_ids = self.collect_leaf_descendant_ids(node_id);
            for id in &descendant_ids {
                if let Some(mesh) = evaluator::evaluate_node(id, &self.recipe, &self.node_hashes) {
                    self.evaluated.meshes.insert(id.clone(), mesh);
                    let matrices = evaluator::compute_world_matrices(&self.recipe.scene, glam::DMat4::IDENTITY);
                    if let Some(node) = evaluator::find_node(&self.recipe.scene, id) {
                        let h = evaluator::combined_hash(node, &matrices);
                        if h != 0 {
                            self.node_hashes.insert(id.clone(), h);
                        }
                    }
                }
            }
            return;
        }

        if let Some(mesh) =
            evaluator::evaluate_node(node_id, &self.recipe, &self.node_hashes)
        {
            self.evaluated.meshes.insert(node_id.to_string(), mesh);
            // Update the combined hash (geometry + world matrix) so subsequent
            // evaluate_all() skips unchanged nodes and reacts to ancestor transforms.
            let matrices = evaluator::compute_world_matrices(&self.recipe.scene, glam::DMat4::IDENTITY);
            if let Some(node) = evaluator::find_node(&self.recipe.scene, node_id) {
                let h = evaluator::combined_hash(node, &matrices);
                if h != 0 {
                    self.node_hashes.insert(node_id.to_string(), h);
                }
            }
        }
    }

    /// Collect IDs of all non-container descendants of a node.
    fn collect_leaf_descendant_ids(&self, node_id: &str) -> Vec<String> {
        let mut ids = Vec::new();
        let mut work = vec![node_id.to_string()];
        while let Some(id) = work.pop() {
            if let Some(node) = evaluator::find_node(&self.recipe.scene, &id) {
                let is_container = matches!(
                    node.node_type,
                    kpe_schema::geometry::GeometryNodeType::Compound
                        | kpe_schema::geometry::GeometryNodeType::Assembly(_)
                );
                if !is_container {
                    ids.push(id);
                }
                for child in &node.children {
                    work.push(child.id.clone());
                }
            }
        }
        ids
    }

    pub fn all_node_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();
        evaluator::collect_ids(&self.recipe.scene, &mut ids);
        ids
    }

    /// Extract a `GeometryScene` from the current recipe for parametric commands.
    pub fn to_scene(&self) -> GeometryScene {
        GeometryScene {
            scene: self.recipe.scene.clone(),
            joints: self.recipe.joints.clone(),
        }
    }

    /// Apply a mutated `GeometryScene` back into the recipe.
    pub fn apply_scene(&mut self, gs: GeometryScene) {
        self.recipe.scene = gs.scene;
        self.recipe.joints = gs.joints;
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}


