//! Thin wrapper around `kpe-geometry` plane math.
//!
//! All core logic has moved to `kpe_geometry::sketch::plane`.
//! This module provides Bevy-typed convenience re-exports.

use bevy::prelude::*;
use bevy::math::Dir3;
use kpe_schema::geometry::{GeometryNode, SketchDef, SketchPlane};

/// Convert 2D sketch coordinates to 3D Bevy space.
pub fn to_3d(x: f64, y: f64, plane: &SketchPlane) -> Vec3 {
    let d = kpe_geometry::sketch::to_3d(x, y, plane);
    Vec3::new(d.x as f32, d.y as f32, d.z as f32)
}

/// Project a Bevy 3D position back to 2D sketch coordinates.
pub fn to_2d(pos: Vec3, plane: &SketchPlane) -> (f64, f64) {
    kpe_geometry::sketch::to_2d(
        glam::DVec3::new(pos.x as f64, pos.y as f64, pos.z as f64),
        plane,
    )
}

/// Get the unit normal vector for a sketch plane as a Bevy `Dir3`.
pub fn sketch_plane_normal(plane: &SketchPlane) -> Dir3 {
    let d = kpe_geometry::sketch::plane_normal(plane);
    Dir3::new(Vec3::new(d.x as f32, d.y as f32, d.z as f32)).unwrap_or(Dir3::Z)
}

/// Compute an orthonormal basis for a circle on a plane.
pub fn circle_basis(normal: Dir3) -> (Vec3, Vec3) {
    let n = glam::DVec3::new(normal.x as f64, normal.y as f64, normal.z as f64);
    let (r, f) = kpe_geometry::sketch::circle_basis(n);
    (
        Vec3::new(r.x as f32, r.y as f32, r.z as f32),
        Vec3::new(f.x as f32, f.y as f32, f.z as f32),
    )
}

/// Extract the `SketchDef` from a geometry node tree.
pub fn get_sketch_def_local(node: &GeometryNode, target: &str) -> Option<SketchDef> {
    kpe_geometry::sketch::get_sketch_def_local(node, target)
}
