use glam::DVec3;
use kpe_schema::geometry::{GeometryNode, GeometryNodeType, SketchDef, SketchPlane};

/// Convert 2D sketch coordinates to 3D space based on the sketch plane.
pub fn to_3d(x: f64, y: f64, plane: &SketchPlane) -> DVec3 {
    match plane {
        SketchPlane::XY => DVec3::new(x, y, 0.0),
        SketchPlane::XZ => DVec3::new(x, 0.0, y),
        SketchPlane::YZ => DVec3::new(0.0, x, y),
    }
}

/// Project a 3D position back to 2D sketch coordinates.
pub fn to_2d(pos: DVec3, plane: &SketchPlane) -> (f64, f64) {
    match plane {
        SketchPlane::XY => (pos.x, pos.y),
        SketchPlane::XZ => (pos.x, pos.z),
        SketchPlane::YZ => (pos.y, pos.z),
    }
}

/// Get the unit normal vector for a sketch plane.
pub fn plane_normal(plane: &SketchPlane) -> DVec3 {
    match plane {
        SketchPlane::XY => DVec3::Z,
        SketchPlane::XZ => DVec3::Y,
        SketchPlane::YZ => DVec3::X,
    }
}

/// Compute an orthonormal basis (right, forward) for a circle on a plane.
///
/// Given a plane normal, returns two perpendicular vectors that span the plane.
pub fn circle_basis(normal: DVec3) -> (DVec3, DVec3) {
    let ref_vec = if normal.x.abs() > 0.9 {
        DVec3::Y
    } else {
        DVec3::X
    };
    let right = normal.cross(ref_vec).normalize();
    let forward = normal.cross(right).normalize();
    (right, forward)
}

/// Extract the `SketchDef` from a geometry node tree by target ID.
pub fn get_sketch_def_local(node: &GeometryNode, target: &str) -> Option<SketchDef> {
    if node.id == target {
        if let GeometryNodeType::Sketch(ref def) = node.node_type {
            return Some(def.clone());
        }
        return None;
    }
    for child in &node.children {
        if let result @ Some(_) = get_sketch_def_local(child, target) {
            return result;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_3d_xy() {
        let p = to_3d(2.0, 3.0, &SketchPlane::XY);
        assert_eq!(p, DVec3::new(2.0, 3.0, 0.0));
    }

    #[test]
    fn test_to_3d_xz() {
        let p = to_3d(2.0, 3.0, &SketchPlane::XZ);
        assert_eq!(p, DVec3::new(2.0, 0.0, 3.0));
    }

    #[test]
    fn test_to_2d_roundtrip() {
        let pos = DVec3::new(1.5, 2.5, 0.0);
        let (x, y) = to_2d(pos, &SketchPlane::XY);
        assert!((x - 1.5).abs() < 1e-10);
        assert!((y - 2.5).abs() < 1e-10);
    }

    #[test]
    fn test_plane_normal() {
        assert_eq!(plane_normal(&SketchPlane::XY), DVec3::Z);
        assert_eq!(plane_normal(&SketchPlane::XZ), DVec3::Y);
        assert_eq!(plane_normal(&SketchPlane::YZ), DVec3::X);
    }

    #[test]
    fn test_circle_basis_orthogonal() {
        let normal = DVec3::new(0.0, 0.0, 1.0);
        let (right, forward) = circle_basis(normal);
        assert!((right.dot(forward)).abs() < 1e-10);
        assert!((right.dot(normal)).abs() < 1e-10);
    }
}
