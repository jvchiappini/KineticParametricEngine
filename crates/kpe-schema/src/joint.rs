use serde::{Deserialize, Serialize};

/// Attachment frame for a joint.
///
/// Defines where and how the joint connects to a body (parent or child).
/// When `current_values` are all zero, the parent frame and child frame
/// are aligned in world space.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct JointFrame {
    /// Position offset from the node's origin (in node-local space).
    pub position: [f64; 3],
    /// Orientation as Euler angles in degrees (applied Z-Y-X).
    pub orientation: [f64; 3],
}

impl Default for JointFrame {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            orientation: [0.0; 3],
        }
    }
}

/// Kinematic joint type with type-specific parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JointType {
    /// 1 rotational DOF around `axis`.
    Revolute { axis: [f64; 3] },
    /// 1 translational DOF along `axis`.
    Prismatic { axis: [f64; 3] },
    /// 2 DOF: rotation + translation along the same `axis`.
    Cylindrical { axis: [f64; 3] },
    /// 2 rotational DOF around two perpendicular axes.
    Universal { axis1: [f64; 3], axis2: [f64; 3] },
    /// 3 rotational DOF (ball-and-socket joint).
    Ball,
    /// 3 DOF: translation in a plane + rotation about the normal.
    Planar { normal: [f64; 3] },
    /// 1 DOF: rotation coupled with translation via `pitch` (mm/rev).
    Screw { axis: [f64; 3], pitch: f64 },
    /// Rigid connection, 0 DOF.
    Fixed,
    /// Full 6 DOF (position + orientation).
    SixDOF,
}

/// Per-DOF motion limits with optional spring behaviour.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DofLimits {
    /// Hard lower bound.
    pub min: f64,
    /// Hard upper bound.
    pub max: f64,
    /// Spring stiffness (N/m or N·m/rad). `None` = rigid stop.
    pub stiffness: Option<f64>,
    /// Damping coefficient (N·s/m or N·m·s/rad).
    pub damping: Option<f64>,
}

impl Default for DofLimits {
    fn default() -> Self {
        Self {
            min: -180.0,
            max: 180.0,
            stiffness: None,
            damping: None,
        }
    }
}

/// Combined motion limits for a joint.
///
/// Not all fields are used by every joint type:
/// - 1-DOF joints use `primary` only.
/// - 2-DOF joints (Cylindrical, Universal) use `primary` and `secondary`.
/// - 3-DOF joints (Ball, Planar) use all three.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointLimits {
    /// Limits for the first (primary) degree of freedom.
    pub primary: DofLimits,
    /// Limits for the second degree of freedom (optional).
    pub secondary: Option<DofLimits>,
    /// Limits for the third degree of freedom (optional).
    pub tertiary: Option<DofLimits>,
}

impl Default for JointLimits {
    fn default() -> Self {
        Self {
            primary: DofLimits::default(),
            secondary: None,
            tertiary: None,
        }
    }
}

/// Convenience type: a variable-length list of joint DOF values.
///
/// Semantics depend on `JointType`:
/// | JointType        | current_values.len | Meaning                                                                  |
/// |------------------|--------------------|--------------------------------------------------------------------------|
/// | Revolute         | 1                  | `[angle_deg]`                                                            |
/// | Prismatic        | 1                  | `[displacement]`                                                         |
/// | Cylindrical      | 2                  | `[angle_deg, displacement]`                                              |
/// | Universal        | 2                  | `[angle1_deg, angle2_deg]`                                               |
/// | Ball             | 3                  | `[roll_deg, pitch_deg, yaw_deg]` (Z-Y-X Euler)                          |
/// | Planar           | 3                  | `[dx, dy, rotation_deg]`                                                 |
/// | Screw            | 1                  | `[rotation_deg]` (displacement = pitch × rotation_deg / 360)            |
/// | Fixed            | 0                  | —                                                                        |
/// | SixDOF           | 6                  | `[x, y, z, rx_deg, ry_deg, rz_deg]`                                     |
pub type JointValues = Vec<f64>;

/// A kinematic joint connecting two scene nodes.
///
/// `parent_id` and `child_id` reference nodes in the scene tree.
/// The joint defines how the child can move relative to the parent.
/// When `child_id` points to a container (Compound / JointGroup / Assembly),
/// the joint matrix propagates to **all** descendant leaf nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Joint {
    /// Unique identifier for this joint.
    pub id: String,
    /// The type of kinematic constraint.
    pub joint_type: JointType,
    /// ID of the parent body node.
    pub parent_id: String,
    /// ID of the child body node (may be a container for group joints).
    pub child_id: String,
    /// Attachment frame on the parent body (in parent's local space).
    pub parent_frame: JointFrame,
    /// Attachment frame on the child body (in child's local space).
    pub child_frame: JointFrame,
    /// Motion limits per degree of freedom.
    pub limits: Option<JointLimits>,
    /// Current DOF values (interpretation depends on `joint_type`).
    pub current_values: JointValues,
}

impl Joint {
    /// Create a simple revolute joint between `parent_id` and `child_id`.
    pub fn revolute(
        id: &str,
        parent_id: &str,
        child_id: &str,
        axis: [f64; 3],
    ) -> Self {
        Self {
            id: id.to_string(),
            joint_type: JointType::Revolute { axis },
            parent_id: parent_id.to_string(),
            child_id: child_id.to_string(),
            parent_frame: JointFrame::default(),
            child_frame: JointFrame::default(),
            limits: None,
            current_values: vec![0.0],
        }
    }

    /// Create a simple prismatic joint between `parent_id` and `child_id`.
    pub fn prismatic(
        id: &str,
        parent_id: &str,
        child_id: &str,
        axis: [f64; 3],
    ) -> Self {
        Self {
            id: id.to_string(),
            joint_type: JointType::Prismatic { axis },
            parent_id: parent_id.to_string(),
            child_id: child_id.to_string(),
            parent_frame: JointFrame::default(),
            child_frame: JointFrame::default(),
            limits: None,
            current_values: vec![0.0],
        }
    }

    /// Create a fixed joint (rigid connection) between two nodes.
    pub fn fixed(id: &str, parent_id: &str, child_id: &str) -> Self {
        Self {
            id: id.to_string(),
            joint_type: JointType::Fixed,
            parent_id: parent_id.to_string(),
            child_id: child_id.to_string(),
            parent_frame: JointFrame::default(),
            child_frame: JointFrame::default(),
            limits: None,
            current_values: vec![],
        }
    }

    /// Return the number of degrees of freedom for this joint type.
    pub fn dof_count(&self) -> usize {
        match self.joint_type {
            JointType::Revolute { .. }
            | JointType::Prismatic { .. }
            | JointType::Screw { .. } => 1,
            JointType::Cylindrical { .. }
            | JointType::Universal { .. } => 2,
            JointType::Ball { .. }
            | JointType::Planar { .. } => 3,
            JointType::Fixed => 0,
            JointType::SixDOF => 6,
        }
    }

    /// Clamp `current_values` to the defined limits (if any).
    pub fn clamp_values(&mut self) {
        let Some(ref limits) = self.limits else { return };

        // Primary DOF
        if let Some(first) = self.current_values.first_mut() {
            *first = first.clamp(limits.primary.min, limits.primary.max);
        }

        // Secondary DOF
        if let (Some(second), Some(sec_limits)) =
            (self.current_values.get_mut(1), limits.secondary)
        {
            *second = second.clamp(sec_limits.min, sec_limits.max);
        }

        // Tertiary DOF
        if let (Some(third), Some(ter_limits)) =
            (self.current_values.get_mut(2), limits.tertiary)
        {
            *third = third.clamp(ter_limits.min, ter_limits.max);
        }
    }

    /// Return the primary DOF value (first element or 0.0).
    pub fn primary_value(&self) -> f64 {
        self.current_values.first().copied().unwrap_or(0.0)
    }
}

/// Runtime state of a joint for animation / simulation purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointState {
    /// Which joint this state belongs to.
    pub joint_id: String,
    /// Current value(s) of the joint (mirrors `Joint.current_values`).
    pub current_values: JointValues,
    /// Velocity for each DOF.
    pub velocities: Vec<f64>,
    /// Acceleration for each DOF.
    pub accelerations: Vec<f64>,
}

impl JointState {
    pub fn new(joint: &Joint) -> Self {
        let dof = joint.dof_count();
        Self {
            joint_id: joint.id.clone(),
            current_values: joint.current_values.clone(),
            velocities: vec![0.0; dof],
            accelerations: vec![0.0; dof],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_revolute_joint_defaults() {
        let j = Joint::revolute("j1", "parent", "child", [0.0, 1.0, 0.0]);
        assert_eq!(j.id, "j1");
        assert_eq!(j.dof_count(), 1);
        assert_eq!(j.primary_value(), 0.0);
    }

    #[test]
    fn test_prismatic_joint() {
        let j = Joint::prismatic("j1", "parent", "child", [1.0, 0.0, 0.0]);
        assert_eq!(j.dof_count(), 1);
        assert_eq!(j.joint_type, JointType::Prismatic { axis: [1.0, 0.0, 0.0] });
    }

    #[test]
    fn test_fixed_joint() {
        let j = Joint::fixed("j_fix", "parent", "child");
        assert_eq!(j.dof_count(), 0);
        assert!(j.current_values.is_empty());
    }

    #[test]
    fn test_clamp_values_with_limits() {
        let mut j = Joint::revolute("j1", "a", "b", [0.0, 0.0, 1.0]);
        j.current_values = vec![200.0];
        j.limits = Some(JointLimits {
            primary: DofLimits {
                min: -90.0,
                max: 90.0,
                stiffness: None,
                damping: None,
            },
            secondary: None,
            tertiary: None,
        });
        j.clamp_values();
        assert_eq!(j.primary_value(), 90.0);
    }

    #[test]
    fn test_dof_counts() {
        assert_eq!(Joint::revolute("r", "a", "b", [0.0, 0.0, 1.0]).dof_count(), 1);
        assert_eq!(Joint::prismatic("p", "a", "b", [1.0, 0.0, 0.0]).dof_count(), 1);

        let cyl = Joint {
            id: "c".into(),
            joint_type: JointType::Cylindrical { axis: [0.0, 1.0, 0.0] },
            parent_id: "a".into(),
            child_id: "b".into(),
            parent_frame: JointFrame::default(),
            child_frame: JointFrame::default(),
            limits: None,
            current_values: vec![0.0, 0.0],
        };
        assert_eq!(cyl.dof_count(), 2);

        let ball = Joint {
            id: "b".into(),
            joint_type: JointType::Ball,
            parent_id: "a".into(),
            child_id: "b".into(),
            parent_frame: JointFrame::default(),
            child_frame: JointFrame::default(),
            limits: None,
            current_values: vec![0.0; 3],
        };
        assert_eq!(ball.dof_count(), 3);

        assert_eq!(Joint::fixed("f", "a", "b").dof_count(), 0);
    }

    #[test]
    fn test_joint_frame_default() {
        let f = JointFrame::default();
        assert_eq!(f.position, [0.0; 3]);
        assert_eq!(f.orientation, [0.0; 3]);
    }

    #[test]
    fn test_joint_state_from_joint() {
        let j = Joint::revolute("j1", "p", "c", [0.0, 1.0, 0.0]);
        let state = JointState::new(&j);
        assert_eq!(state.joint_id, "j1");
        assert_eq!(state.velocities.len(), 1);
        assert_eq!(state.accelerations.len(), 1);
    }
}
