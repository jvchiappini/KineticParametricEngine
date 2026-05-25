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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::*;
    use crate::block::*;

    #[test]
    fn test_recipe_default() {
        let recipe = KPERecipe::default();
        assert_eq!(recipe.version, "0.1.0");
        assert_eq!(recipe.metadata.name, "Untitled");
        assert!(recipe.blocks.is_empty());
        assert!(recipe.joints.is_empty());
    }

    #[test]
    fn test_recipe_serialize_roundtrip() {
        let recipe = KPERecipe::default();
        let json = serde_json::to_string(&recipe).unwrap();
        let deserialized: KPERecipe = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.version, recipe.version);
        assert_eq!(deserialized.metadata.name, recipe.metadata.name);
    }

    #[test]
    fn test_recipe_with_block() {
        let mut recipe = KPERecipe::default();
        let mut params = std::collections::HashMap::new();
        params.insert("width".to_string(), ParamSchema {
            param_type: ParamType::Number,
            default: serde_json::json!(600.0),
            min: Some(200.0),
            max: Some(1200.0),
            unit: Some("mm".to_string()),
            options: None,
        });
        recipe.blocks.insert("test_block".to_string(), BlockDefinition {
            id: "test_block".to_string(),
            label: "Test".to_string(),
            params,
            variables: std::collections::HashMap::new(),
            rules: vec![],
            joints: std::collections::HashMap::new(),
            geometry: None,
            material: None,
        });
        assert_eq!(recipe.blocks.len(), 1);
    }

    #[test]
    fn test_scene_node_types() {
        let box_node = GeometryNode {
            id: "box1".to_string(),
            node_type: GeometryNodeType::Box(BoxDef { width: 10.0, height: 20.0, depth: 30.0 }),
            transform: None,
            children: vec![],
            operations: vec![],
        };
        match box_node.node_type {
            GeometryNodeType::Box(b) => {
                assert_eq!(b.width, 10.0);
                assert_eq!(b.height, 20.0);
                assert_eq!(b.depth, 30.0);
            }
            _ => panic!("Wrong node type"),
        }
    }

    #[test]
    fn test_material_defaults() {
        let mat = crate::material::ProceduralMaterial {
            id: "wood".to_string(),
            base: crate::material::MaterialBase {
                color: "#8B5E3C".to_string(),
                roughness: 0.8,
                metalness: 0.0,
            },
            uv_mode: crate::material::UvMode::WorldScale,
            uv_scale: [1.0, 1.0],
            instance_vars: std::collections::HashMap::new(),
            layers: vec![],
        };
        assert_eq!(mat.base.color, "#8B5E3C");
    }
}
