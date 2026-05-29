use bevy::prelude::*;
use bevy::math::Dir3;
use kpe_schema::geometry::{GeometryNode, GeometryNodeType, SketchDef, SketchPlane};

pub fn to_3d(x: f64, y: f64, plane: &SketchPlane) -> Vec3 {
    match plane {
        SketchPlane::XY => Vec3::new(x as f32, y as f32, 0.0),
        SketchPlane::XZ => Vec3::new(x as f32, 0.0, y as f32),
        SketchPlane::YZ => Vec3::new(0.0, x as f32, y as f32),
    }
}

pub fn to_2d(pos: Vec3, plane: &SketchPlane) -> (f64, f64) {
    match plane {
        SketchPlane::XY => (pos.x as f64, pos.y as f64),
        SketchPlane::XZ => (pos.x as f64, pos.z as f64),
        SketchPlane::YZ => (pos.y as f64, pos.z as f64),
    }
}

pub fn sketch_plane_normal(plane: &SketchPlane) -> Dir3 {
    match plane {
        SketchPlane::XY => Dir3::Z,
        SketchPlane::XZ => Dir3::Y,
        SketchPlane::YZ => Dir3::X,
    }
}

pub fn circle_basis(normal: Dir3) -> (Vec3, Vec3) {
    let n = normal.as_vec3();
    let ref_vec = if n.abs().x > 0.9 { Vec3::Y } else { Vec3::X };
    let right = n.cross(ref_vec).normalize();
    let forward = n.cross(right).normalize();
    (right, forward)
}

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
