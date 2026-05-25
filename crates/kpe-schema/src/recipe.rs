use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::block::BlockDefinition;
use crate::geometry::GeometryNode;
use crate::joint::Joint;
use crate::constraint::Constraint;
use crate::material::ProceduralMaterial;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KPERecipe {
    pub version: String,
    pub metadata: RecipeMetadata,
    pub blocks: HashMap<String, BlockDefinition>,
    pub scene: GeometryNode,
    pub joints: Vec<Joint>,
    pub constraints: Vec<Constraint>,
    pub materials: HashMap<String, ProceduralMaterial>,
    pub precision: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeMetadata {
    pub name: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub created_at: Option<String>,
    pub tags: Vec<String>,
}

impl Default for KPERecipe {
    fn default() -> Self {
        Self {
            version: "0.1.0".to_string(),
            metadata: RecipeMetadata {
                name: "Untitled".to_string(),
                author: None,
                description: None,
                created_at: None,
                tags: vec![],
            },
            blocks: HashMap::new(),
            scene: GeometryNode {
                id: "root".to_string(),
                node_type: crate::geometry::GeometryNodeType::Compound,
                transform: None,
                children: vec![],
                operations: vec![],
            },
            joints: vec![],
            constraints: vec![],
            materials: HashMap::new(),
            precision: Some(1e-6),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedRecipe {
    pub recipe: KPERecipe,
    pub resolved_params: HashMap<String, HashMap<String, f64>>,
    pub resolved_variables: HashMap<String, HashMap<String, f64>>,
    pub active_rules: Vec<String>,
}
