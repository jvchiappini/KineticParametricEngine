use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceduralMaterial {
    pub id: String,
    pub base: MaterialBase,
    pub uv_mode: UvMode,
    pub uv_scale: [f64; 2],
    pub instance_vars: HashMap<String, InstanceVarDef>,
    pub layers: Vec<MaterialLayer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialBase {
    pub color: String,
    pub roughness: f64,
    pub metalness: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UvMode {
    WorldScale,
    ObjectRelative,
    Planar,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceVarDef {
    pub var_type: InstanceVarType,
    pub range: Option<[i64; 2]>,
    pub default: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstanceVarType {
    RandomInt,
    String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialLayer {
    pub layer_type: LayerType,
    pub params: HashMap<String, serde_json::Value>,
    pub visible_expr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayerType {
    WoodGrain,
    TextOverlay,
    Noise,
    Concrete,
    Solid,
}
