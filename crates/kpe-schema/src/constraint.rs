use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: String,
    pub constraint_type: ConstraintType,
    pub entities: Vec<String>,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    Distance,
    Angle,
    Coincident,
    Parallel,
    Perpendicular,
    Tangent,
    Concentric,
    Horizontal,
    Vertical,
    Equal,
    Fix,
}
