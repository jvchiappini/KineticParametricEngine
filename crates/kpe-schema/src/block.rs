use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::geometry::GeometryNode;
use crate::joint::{JointType, JointLimits};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamSchema {
    pub param_type: ParamType,
    pub default: serde_json::Value,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub unit: Option<String>,
    pub options: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParamType {
    Number,
    Integer,
    Boolean,
    Enum,
    String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDefinition {
    pub id: String,
    pub label: String,
    pub params: HashMap<String, ParamSchema>,
    pub variables: HashMap<String, String>,
    pub rules: Vec<Rule>,
    pub joints: HashMap<String, JointDef>,
    pub geometry: Option<GeometryNode>,
    pub material: Option<MaterialRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub when: String,
    pub then: Vec<RuleAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    AddChild(ChildDef),
    AddOperation(OperationDef),
    SetParam(SetParamAction),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildDef {
    pub id: String,
    pub block_type: String,
    pub params: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationDef {
    pub op_type: String,
    pub tool: GeometryNode,
    pub array: Option<ArrayDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetParamAction {
    pub param: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrayDef {
    pub pattern: ArrayPattern,
    pub axis: String,
    pub count: serde_json::Value,
    pub spacing: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArrayPattern {
    Linear,
    Circular,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointDef {
    pub joint_type: JointType,
    pub axis: [f64; 3],
    pub limits: Option<JointLimits>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialRef {
    pub material_id: String,
    pub instance_vars: HashMap<String, serde_json::Value>,
}
