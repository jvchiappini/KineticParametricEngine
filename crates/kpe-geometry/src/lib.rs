pub mod brep;
pub mod csg;
pub mod transform;
pub mod joint;
pub mod mesh;
pub mod sketch;
pub mod extrude;
pub mod predicates;
pub mod intersection;
pub mod bvh;
pub mod classify;
pub mod stitch;
pub mod split;
pub mod face;
pub mod push_pull;
pub mod evaluator;

pub use brep::BRepKernel;
pub use csg::CsgKernel;
pub use transform::TransformEngine;
pub use joint::JointEngine;
pub use mesh::MeshBuilder;
pub use mesh::build_mesh_from_node;
pub use sketch::SketchEngine;
pub use sketch::SketchDocument;
pub use sketch::triangulate_contour;
pub use sketch::entities::{Point, Line, Arc, Circle, EntityId};
pub use sketch::constraints::Constraint;
pub use sketch::solver::Solver;
pub use sketch::inference::{InferenceEngine, SnapResult};
pub use sketch::face_tree::FaceTreeBuilder;
pub use sketch::polar_tracking::{PolarSnap, PolarSnapResult};
pub use sketch::dynamic_input::{DynamicInput, DynamicResult};
pub use sketch::osnap::OsnapEngine;
pub use face::{Face, detect_faces, extrude_face};
pub use push_pull::extrude_face as extrude_triangle;
pub use evaluator::{
    evaluate_scene, evaluate_node, compute_world_matrices, hash_geometry_node,
    combined_hash, build_mesh_with_joints, find_node, find_parent, find_parent_mut,
    collect_ids, SceneGeometry,
};
pub use predicates::*;
pub use intersection::triangle_triangle_intersection;
pub use bvh::BVH;
pub use classify::classify_point_against_mesh;
pub use stitch::Stitcher;
