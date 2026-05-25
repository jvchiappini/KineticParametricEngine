use glam::{DMat4, DVec3};
use kpe_schema::geometry::TransformOp;

pub struct TransformEngine;

impl TransformEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn compute_matrix(&self, transform: &TransformOp) -> DMat4 {
        let mut matrix = DMat4::IDENTITY;

        if let Some(trans) = &transform.translation {
            matrix = DMat4::from_translation(DVec3::new(trans[0], trans[1], trans[2]));
        }

        if let Some(rot) = &transform.rotation {
            let rx = DMat4::from_rotation_x(rot[0].to_radians());
            let ry = DMat4::from_rotation_y(rot[1].to_radians());
            let rz = DMat4::from_rotation_z(rot[2].to_radians());
            matrix = matrix * rz * ry * rx;
        }

        if let Some(scale) = &transform.scale {
            matrix = matrix * DMat4::from_scale(DVec3::new(scale[0], scale[1], scale[2]));
        }

        matrix
    }

    pub fn accumulate_world_matrix(
        &self,
        local_matrix: DMat4,
        parent_world: DMat4,
    ) -> DMat4 {
        parent_world * local_matrix
    }

    pub fn identity(&self) -> DMat4 {
        DMat4::IDENTITY
    }
}

impl Default for TransformEngine {
    fn default() -> Self {
        Self::new()
    }
}
