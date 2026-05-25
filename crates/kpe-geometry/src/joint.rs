use glam::DMat4;
use kpe_schema::joint::{Joint, JointType};

pub struct JointEngine;

impl JointEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn compute_joint_matrix(&self, joint: &Joint) -> DMat4 {
        match joint.joint_type {
            JointType::Revolute => {
                let pivot = glam::DVec3::new(joint.pivot[0], joint.pivot[1], joint.pivot[2]);
                let axis = glam::DVec3::new(joint.axis[0], joint.axis[1], joint.axis[2]);
                let angle_rad = joint.current_value.to_radians();

                let to_pivot = DMat4::from_translation(pivot);
                let rotation = DMat4::from_axis_angle(axis, angle_rad);
                let from_pivot = DMat4::from_translation(-pivot);

                to_pivot * rotation * from_pivot
            }
            JointType::Prismatic => {
                let axis = glam::DVec3::new(joint.axis[0], joint.axis[1], joint.axis[2]);
                DMat4::from_translation(axis * joint.current_value)
            }
            JointType::Fixed => DMat4::IDENTITY,
            JointType::Ball => {
                let pivot = glam::DVec3::new(joint.pivot[0], joint.pivot[1], joint.pivot[2]);
                let angle_rad = joint.current_value.to_radians();
                let axis = glam::DVec3::new(joint.axis[0], joint.axis[1], joint.axis[2]);

                let to_pivot = DMat4::from_translation(pivot);
                let rotation = DMat4::from_axis_angle(axis, angle_rad);
                let from_pivot = DMat4::from_translation(-pivot);

                to_pivot * rotation * from_pivot
            }
        }
    }

    pub fn clamp_joint_value(&self, joint: &Joint) -> f64 {
        match &joint.limits {
            Some(limits) => joint.current_value.clamp(limits.min, limits.max),
            None => joint.current_value,
        }
    }
}

impl Default for JointEngine {
    fn default() -> Self {
        Self::new()
    }
}
