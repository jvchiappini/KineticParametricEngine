use kpe_schema::geometry::{GeometryNode, PivotTransform, TransformOp};

use crate::scene::GeometryScene;
use super::Command;
use super::find_node_mut;

// ── AddPivotCommand ───────────────────────────────────────────────

/// Command to add a new pivot-relative transform step to a node.
pub struct AddPivotCommand {
    pub node_id: String,
    pub pivot: PivotTransform,
}

impl Command for AddPivotCommand {
    fn execute(&mut self, scene: &mut GeometryScene) {
        if let Some(node) = find_node_mut(&mut scene.scene, &self.node_id) {
            let tf = node.transform.get_or_insert_with(|| TransformOp {
                translation: None,
                rotation: None,
                scale: None,
                pivots: Vec::new(),
            });
            tf.pivots.push(self.pivot.clone());
        }
    }

    fn undo(&mut self, scene: &mut GeometryScene) {
        if let Some(node) = find_node_mut(&mut scene.scene, &self.node_id) {
            if let Some(tf) = &mut node.transform {
                tf.pivots.retain(|p| p.id != self.pivot.id);
            }
        }
    }

    fn description(&self) -> &str {
        "Add Pivot"
    }
}

// ── RemovePivotCommand ────────────────────────────────────────────

/// Command to remove a pivot-relative transform step by its ID.
pub struct RemovePivotCommand {
    pub node_id: String,
    pub pivot_id: String,
    pub old_pivot: Option<PivotTransform>,
}

impl Command for RemovePivotCommand {
    fn execute(&mut self, scene: &mut GeometryScene) {
        if let Some(node) = find_node_mut(&mut scene.scene, &self.node_id) {
            if let Some(tf) = &mut node.transform {
                let idx = tf.pivots.iter().position(|p| p.id == self.pivot_id);
                if let Some(i) = idx {
                    self.old_pivot = Some(tf.pivots.remove(i));
                }
            }
        }
    }

    fn undo(&mut self, scene: &mut GeometryScene) {
        if let Some(old) = &self.old_pivot {
            if let Some(node) = find_node_mut(&mut scene.scene, &self.node_id) {
                let tf = node.transform.get_or_insert_with(|| TransformOp {
                    translation: None,
                    rotation: None,
                    scale: None,
                    pivots: Vec::new(),
                });
                tf.pivots.push(old.clone());
            }
        }
    }

    fn description(&self) -> &str {
        "Remove Pivot"
    }
}

// ── SetPivotTransformCommand ──────────────────────────────────────

/// Command to update a single numeric field on a pivot transform.
pub struct SetPivotTransformCommand {
    pub node_id: String,
    pub pivot_id: String,
    pub field: PivotField,
    pub old_value: f64,
    pub new_value: f64,
}

pub enum PivotField {
    PivotX,
    PivotY,
    PivotZ,
    TranslationX,
    TranslationY,
    TranslationZ,
    RotationX,
    RotationY,
    RotationZ,
    ScaleX,
    ScaleY,
    ScaleZ,
}

impl Command for SetPivotTransformCommand {
    fn execute(&mut self, scene: &mut GeometryScene) {
        apply_pivot_field(&mut scene.scene, &self.node_id, &self.pivot_id, &self.field, self.new_value);
    }

    fn undo(&mut self, scene: &mut GeometryScene) {
        apply_pivot_field(&mut scene.scene, &self.node_id, &self.pivot_id, &self.field, self.old_value);
    }

    fn description(&self) -> &str {
        "Set Pivot Field"
    }
}

fn apply_pivot_field(
    node: &mut GeometryNode,
    target: &str,
    pivot_id: &str,
    field: &PivotField,
    value: f64,
) {
    if node.id != target {
        for child in &mut node.children {
            apply_pivot_field(child, target, pivot_id, field, value);
        }
        return;
    }

    if let Some(tf) = &mut node.transform {
        if let Some(pv) = tf.pivots.iter_mut().find(|p| p.id == pivot_id) {
            match field {
                PivotField::PivotX => pv.pivot[0] = value,
                PivotField::PivotY => pv.pivot[1] = value,
                PivotField::PivotZ => pv.pivot[2] = value,

                PivotField::TranslationX => {
                    let mut t = pv.translation.unwrap_or([0.0; 3]);
                    t[0] = value;
                    pv.translation = Some(t);
                }
                PivotField::TranslationY => {
                    let mut t = pv.translation.unwrap_or([0.0; 3]);
                    t[1] = value;
                    pv.translation = Some(t);
                }
                PivotField::TranslationZ => {
                    let mut t = pv.translation.unwrap_or([0.0; 3]);
                    t[2] = value;
                    pv.translation = Some(t);
                }

                PivotField::RotationX => {
                    let mut r = pv.rotation.unwrap_or([0.0; 3]);
                    r[0] = value;
                    pv.rotation = Some(r);
                }
                PivotField::RotationY => {
                    let mut r = pv.rotation.unwrap_or([0.0; 3]);
                    r[1] = value;
                    pv.rotation = Some(r);
                }
                PivotField::RotationZ => {
                    let mut r = pv.rotation.unwrap_or([0.0; 3]);
                    r[2] = value;
                    pv.rotation = Some(r);
                }

                PivotField::ScaleX => {
                    let mut s = pv.scale.unwrap_or([1.0; 3]);
                    s[0] = value;
                    pv.scale = Some(s);
                }
                PivotField::ScaleY => {
                    let mut s = pv.scale.unwrap_or([1.0; 3]);
                    s[1] = value;
                    pv.scale = Some(s);
                }
                PivotField::ScaleZ => {
                    let mut s = pv.scale.unwrap_or([1.0; 3]);
                    s[2] = value;
                    pv.scale = Some(s);
                }
            }
        }
    }
}

// ── Helpers for generating pivot IDs ──────────────────────────────

/// Generate a unique pivot ID for a given node.
pub fn generate_pivot_id(node_id: &str, existing: &[PivotTransform]) -> String {
    let base = format!("{}/pivot", node_id);
    let count = existing.iter().filter(|p| p.id.starts_with(&base)).count();
    format!("{}_{}", base, count + 1)
}
