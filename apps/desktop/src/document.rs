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
        self.evaluated = sg;
    }

    /// Evaluate a single node by ID.
    pub fn evaluate_node(&mut self, node_id: &str) {
        if let Some(mesh) =
            evaluator::evaluate_node(node_id, &self.recipe, &self.node_hashes)
        {
            self.evaluated.meshes.insert(node_id.to_string(), mesh);
        }
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


