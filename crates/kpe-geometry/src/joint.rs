use glam::{DMat4, DVec3};
use kpe_schema::joint::{Joint, JointFrame, JointType, JointValues};

/// Engine for computing joint transformation matrices.
///
/// Supports all joint types defined in the schema, including frame-aware
/// attachment transforms and group joint propagation.
pub struct JointEngine;

impl JointEngine {
    pub fn new() -> Self {
        Self
    }

    /// Compute the transformation matrix that maps the child body from its
    /// local space to the parent's local space through this joint.
    ///
    /// The formula used is:
    ///
    /// ```text
    /// joint_matrix = T(parent_frame_pos) · R(parent_frame_orient)
    ///               · motion(current_values)
    ///               · R⁻¹(child_frame_orient) · T(-child_frame_pos)
    /// ```
    ///
    /// When all `current_values` are zero, the result aligns the parent and
    /// child frames, producing identity motion.
    pub fn compute_joint_matrix(&self, joint: &Joint) -> DMat4 {
        // Build parent frame transform: position → orientation
        let parent_frame = Self::frame_to_matrix(&joint.parent_frame);
        // Build child frame inverse: orientation⁻¹ → -position
        let child_frame_inv = Self::frame_to_inverse(&joint.child_frame);
        // Build the motion transform from current_values
        let motion = Self::compute_motion(&joint.joint_type, &joint.current_values);

        parent_frame * motion * child_frame_inv
    }

    /// Compute the joint matrix but only the "motion" part (ignoring frames).
    /// Useful for preview / gizmo display.
    pub fn compute_motion_matrix(&self, joint: &Joint) -> DMat4 {
        Self::compute_motion(&joint.joint_type, &joint.current_values)
    }

    /// Clamp the joint's current values to the defined limits.
    pub fn clamp_joint_values(&self, joint: &mut Joint) {
        joint.clamp_values();
    }

    /// Return the axis vector for joint types that have one (Revolute,
    /// Prismatic, Cylindrical, Screw).
    pub fn joint_axis(&self, joint: &Joint) -> Option<DVec3> {
        match joint.joint_type {
            JointType::Revolute { axis }
            | JointType::Prismatic { axis }
            | JointType::Cylindrical { axis }
            | JointType::Screw { axis, .. } => Some(DVec3::new(axis[0], axis[1], axis[2])),
            _ => None,
        }
    }

    // ── Private helpers ──────────────────────────────────────────

    /// Convert a `JointFrame` into a homogeneous 4×4 matrix.
    ///
    /// Order: translate(position) · rotate_z(yaw) · rotate_y(pitch) · rotate_x(roll)
    fn frame_to_matrix(frame: &JointFrame) -> DMat4 {
        let t = DMat4::from_translation(DVec3::new(frame.position[0], frame.position[1], frame.position[2]));
        let r = Self::euler_to_matrix(frame.orientation);
        t * r
    }

    /// Inverse of `frame_to_matrix`: undo rotation, then undo translation.
    fn frame_to_inverse(frame: &JointFrame) -> DMat4 {
        let r_inv = Self::euler_to_matrix(frame.orientation).transpose(); // rotation inverse = transpose
        let t_inv = DMat4::from_translation(-DVec3::new(frame.position[0], frame.position[1], frame.position[2]));
        r_inv * t_inv
    }

    /// Build rotation matrix from Z-Y-X Euler angles (degrees).
    fn euler_to_matrix(euler: [f64; 3]) -> DMat4 {
        let rx = DMat4::from_rotation_x(euler[0].to_radians());
        let ry = DMat4::from_rotation_y(euler[1].to_radians());
        let rz = DMat4::from_rotation_z(euler[2].to_radians());
        rz * ry * rx
    }

    /// Build the motion transform based on joint type and current values.
    fn compute_motion(joint_type: &JointType, values: &JointValues) -> DMat4 {
        match joint_type {
            JointType::Revolute { axis } => {
                let angle_rad = values.first().copied().unwrap_or(0.0).to_radians();
                let ax = DVec3::new(axis[0], axis[1], axis[2]);
                DMat4::from_axis_angle(ax, angle_rad)
            }
            JointType::Prismatic { axis } => {
                let disp = values.first().copied().unwrap_or(0.0);
                let ax = DVec3::new(axis[0], axis[1], axis[2]);
                DMat4::from_translation(ax * disp)
            }
            JointType::Cylindrical { axis } => {
                let angle_rad = values.first().copied().unwrap_or(0.0).to_radians();
                let disp = values.get(1).copied().unwrap_or(0.0);
                let ax = DVec3::new(axis[0], axis[1], axis[2]);
                let rot = DMat4::from_axis_angle(ax, angle_rad);
                let tr = DMat4::from_translation(ax * disp);
                tr * rot
            }
            JointType::Universal { axis1, axis2 } => {
                let a1_rad = values.first().copied().unwrap_or(0.0).to_radians();
                let a2_rad = values.get(1).copied().unwrap_or(0.0).to_radians();
                let ax1 = DVec3::new(axis1[0], axis1[1], axis1[2]);
                let ax2 = DVec3::new(axis2[0], axis2[1], axis2[2]);
                DMat4::from_axis_angle(ax1, a1_rad) * DMat4::from_axis_angle(ax2, a2_rad)
            }
            JointType::Ball => {
                let roll = values.get(0).copied().unwrap_or(0.0).to_radians();
                let pitch = values.get(1).copied().unwrap_or(0.0).to_radians();
                let yaw = values.get(2).copied().unwrap_or(0.0).to_radians();
                DMat4::from_rotation_z(yaw)
                    * DMat4::from_rotation_y(pitch)
                    * DMat4::from_rotation_x(roll)
            }
            JointType::Planar { normal } => {
                let dx = values.get(0).copied().unwrap_or(0.0);
                let dy = values.get(1).copied().unwrap_or(0.0);
                let rot_deg = values.get(2).copied().unwrap_or(0.0);
                let n = DVec3::new(normal[0], normal[1], normal[2]);
                let rot = DMat4::from_axis_angle(n, rot_deg.to_radians());
                let tr = DMat4::from_translation(DVec3::new(dx, dy, 0.0));
                tr * rot
            }
            JointType::Screw { axis, pitch } => {
                let angle_rad = values.first().copied().unwrap_or(0.0).to_radians();
                let ax = DVec3::new(axis[0], axis[1], axis[2]);
                let displacement = pitch * angle_rad / (2.0 * std::f64::consts::PI);
                let rot = DMat4::from_axis_angle(ax, angle_rad);
                let tr = DMat4::from_translation(ax * displacement);
                tr * rot
            }
            JointType::Fixed => DMat4::IDENTITY,
            JointType::SixDOF => {
                let x = values.get(0).copied().unwrap_or(0.0);
                let y = values.get(1).copied().unwrap_or(0.0);
                let z = values.get(2).copied().unwrap_or(0.0);
                let rx = values.get(3).copied().unwrap_or(0.0).to_radians();
                let ry = values.get(4).copied().unwrap_or(0.0).to_radians();
                let rz = values.get(5).copied().unwrap_or(0.0).to_radians();
                let tr = DMat4::from_translation(DVec3::new(x, y, z));
                let rot = DMat4::from_rotation_z(rz) * DMat4::from_rotation_y(ry) * DMat4::from_rotation_x(rx);
                tr * rot
            }
        }
    }
}

impl Default for JointEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kpe_schema::joint::{Joint, JointFrame, JointType};

    fn identity_joint() -> Joint {
        Joint::fixed("test", "parent", "child")
    }

    fn make_joint(joint_type: JointType, values: Vec<f64>) -> Joint {
        Joint {
            id: "test".into(),
            joint_type,
            parent_id: "parent".into(),
            child_id: "child".into(),
            parent_frame: JointFrame::default(),
            child_frame: JointFrame::default(),
            limits: None,
            current_values: values,
        }
    }

    #[test]
    fn test_fixed_is_identity() {
        let engine = JointEngine::new();
        let m = engine.compute_joint_matrix(&identity_joint());
        assert_eq!(m, DMat4::IDENTITY);
    }

    #[test]
    fn test_revolute_at_zero_is_identity() {
        let engine = JointEngine::new();
        let j = make_joint(JointType::Revolute { axis: [0.0, 1.0, 0.0] }, vec![0.0]);
        let m = engine.compute_joint_matrix(&j);
        // At zero angle, motion is identity; with default frames, total is identity
        let diff = m - DMat4::IDENTITY;
        let cols = [diff.x_axis, diff.y_axis, diff.z_axis, diff.w_axis];
        for (ci, col) in cols.iter().enumerate() {
            for ri in 0..4 {
                let val = col[ri];
                assert!(val.abs() < 1e-10,
                    "Revolute at zero: diff[{},{}] = {}", ci, ri, val);
            }
        }
    }

    #[test]
    fn test_revolute_90_degrees() {
        let engine = JointEngine::new();
        let j = make_joint(JointType::Revolute { axis: [0.0, 0.0, 1.0] }, vec![90.0]);
        let m = engine.compute_joint_matrix(&j);
        // Should rotate the local X axis to Y
        let x_rotated = m.transform_point3(DVec3::X);
        assert!((x_rotated - DVec3::Y).length() < 1e-10);
    }

    #[test]
    fn test_prismatic_displacement() {
        let engine = JointEngine::new();
        let j = make_joint(JointType::Prismatic { axis: [1.0, 0.0, 0.0] }, vec![5.0]);
        let m = engine.compute_joint_matrix(&j);
        let origin = m.transform_point3(DVec3::ZERO);
        assert!((origin - DVec3::new(5.0, 0.0, 0.0)).length() < 1e-10);
    }

    #[test]
    fn test_screw_displacement() {
        let engine = JointEngine::new();
        let j = make_joint(JointType::Screw { axis: [0.0, 1.0, 0.0], pitch: 10.0 }, vec![360.0]);
        let m = engine.compute_joint_matrix(&j);
        let origin = m.transform_point3(DVec3::ZERO);
        // One full turn with pitch 10mm → displacement of 10mm along axis
        assert!((origin - DVec3::new(0.0, 10.0, 0.0)).length() < 1e-10);
    }

    #[test]
    fn test_frame_offset() {
        let engine = JointEngine::new();
        let j = Joint {
            id: "test".into(),
            joint_type: JointType::Fixed,
            parent_id: "parent".into(),
            child_id: "child".into(),
            parent_frame: JointFrame {
                position: [2.0, 0.0, 0.0],
                orientation: [0.0; 3],
            },
            child_frame: JointFrame::default(),
            limits: None,
            current_values: vec![],
        };
        let m = engine.compute_joint_matrix(&j);
        let origin = m.transform_point3(DVec3::ZERO);
        // Fixed joint with parent frame at (2,0,0): origin maps to (2,0,0)
        assert!((origin - DVec3::new(2.0, 0.0, 0.0)).length() < 1e-10);
    }

    #[test]
    fn test_clamp_values() {
        let mut j = make_joint(JointType::Prismatic { axis: [1.0, 0.0, 0.0] }, vec![500.0]);
        j.limits = Some(kpe_schema::joint::JointLimits {
            primary: kpe_schema::joint::DofLimits {
                min: -100.0,
                max: 100.0,
                stiffness: None,
                damping: None,
            },
            secondary: None,
            tertiary: None,
        });
        let engine = JointEngine::new();
        engine.clamp_joint_values(&mut j);
        assert_eq!(j.primary_value(), 100.0);
    }

    #[test]
    fn test_joint_axis_extraction() {
        let engine = JointEngine::new();
        let j = make_joint(JointType::Revolute { axis: [0.0, 1.0, 0.0] }, vec![0.0]);
        let axis = engine.joint_axis(&j);
        assert!(axis.is_some());
        assert!((axis.unwrap() - DVec3::Y).length() < 1e-10);

        let fixed = identity_joint();
        assert!(engine.joint_axis(&fixed).is_none());
    }

    #[test]
    fn test_motion_matrix_only() {
        let engine = JointEngine::new();
        let j = make_joint(JointType::Revolute { axis: [0.0, 0.0, 1.0] }, vec![90.0]);
        let motion = engine.compute_motion_matrix(&j);
        let full = engine.compute_joint_matrix(&j);
        // With default frames, motion equals full
        let diff = motion - full;
        let max_elem = diff.x_axis.to_array().iter().chain(
            diff.y_axis.to_array().iter()).chain(
            diff.z_axis.to_array().iter()).chain(
            diff.w_axis.to_array().iter())
            .map(|v| v.abs()).fold(0.0_f64, f64::max);
        assert!(max_elem < 1e-10, "motion ≠ full: max diff = {}", max_elem);
    }
}
