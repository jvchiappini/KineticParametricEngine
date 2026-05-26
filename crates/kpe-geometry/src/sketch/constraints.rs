use serde::{Deserialize, Serialize};
use crate::sketch::entities::EntityId;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ConstraintTag {
    Horizontal,
    Vertical,
    Coincident,
    Fix,
    Distance,
    EqualLength,
    Parallel,
    Perpendicular,
    Midpoint,
    Tangent,
    Radius,
    Angle,
    Collinear,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Constraint {
    Horizontal {
        line: EntityId,
    },
    Vertical {
        line: EntityId,
    },
    Coincident {
        point_a: EntityId,
        point_b: EntityId,
    },
    Fix {
        point: EntityId,
        x: f64,
        y: f64,
    },
    Distance {
        point_a: EntityId,
        point_b: EntityId,
        distance: f64,
    },
    EqualLength {
        line_a: EntityId,
        line_b: EntityId,
    },
    Parallel {
        line_a: EntityId,
        line_b: EntityId,
    },
    Perpendicular {
        line_a: EntityId,
        line_b: EntityId,
    },
    Midpoint {
        point: EntityId,
        line: EntityId,
    },
    Tangent {
        line: EntityId,
        arc: EntityId,
    },
    Radius {
        arc_or_circle: EntityId,
        radius: f64,
    },
    Angle {
        line_a: EntityId,
        line_b: EntityId,
        angle: f64,
    },
    Collinear {
        line_a: EntityId,
        line_b: EntityId,
    },
}

impl Constraint {
    pub fn tag(&self) -> ConstraintTag {
        match self {
            Constraint::Horizontal { .. } => ConstraintTag::Horizontal,
            Constraint::Vertical { .. } => ConstraintTag::Vertical,
            Constraint::Coincident { .. } => ConstraintTag::Coincident,
            Constraint::Fix { .. } => ConstraintTag::Fix,
            Constraint::Distance { .. } => ConstraintTag::Distance,
            Constraint::EqualLength { .. } => ConstraintTag::EqualLength,
            Constraint::Parallel { .. } => ConstraintTag::Parallel,
            Constraint::Perpendicular { .. } => ConstraintTag::Perpendicular,
            Constraint::Midpoint { .. } => ConstraintTag::Midpoint,
            Constraint::Tangent { .. } => ConstraintTag::Tangent,
            Constraint::Radius { .. } => ConstraintTag::Radius,
            Constraint::Angle { .. } => ConstraintTag::Angle,
            Constraint::Collinear { .. } => ConstraintTag::Collinear,
        }
    }
}

pub fn describe_short(c: &Constraint) -> String {
    match c {
        Constraint::Horizontal { .. } => "H".into(),
        Constraint::Vertical { .. } => "V".into(),
        Constraint::Coincident { .. } => "⊙".into(),
        Constraint::Fix { .. } => "📌".into(),
        Constraint::Distance { distance, .. } => format!("{:.1}", distance),
        Constraint::EqualLength { .. } => "=".into(),
        Constraint::Parallel { .. } => "∥".into(),
        Constraint::Perpendicular { .. } => "⟂".into(),
        Constraint::Midpoint { .. } => "━".into(),
        Constraint::Tangent { .. } => "⌓".into(),
        Constraint::Radius { radius, .. } => format!("R{:.1}", radius),
        Constraint::Angle { angle, .. } => format!("{:.1}°", angle.to_degrees()),
        Constraint::Collinear { .. } => "≡".into(),
    }
}
