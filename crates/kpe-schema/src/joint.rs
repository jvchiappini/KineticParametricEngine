use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Joint {
    pub id: String,
    pub joint_type: JointType,
    pub parent_id: String,
    pub child_id: String,
    pub pivot: [f64; 3],
    pub axis: [f64; 3],
    pub limits: Option<JointLimits>,
    pub current_value: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JointType {
    Revolute,
    Prismatic,
    Fixed,
    Ball,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointLimits {
    pub min: f64,
    pub max: f64,
    pub damping: Option<f64>,
    pub stiffness: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointState {
    pub joint_id: String,
    pub current_value: f64,
    pub velocity: f64,
}
