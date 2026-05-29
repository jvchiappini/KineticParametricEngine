use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometryNode {
    pub id: String,
    pub node_type: GeometryNodeType,
    pub transform: Option<TransformOp>,
    pub children: Vec<GeometryNode>,
    pub operations: Vec<CsgOperation>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeometryNodeType {
    Box(BoxDef),
    Cylinder(CylinderDef),
    Sphere(SphereDef),
    Mesh(MeshDef),
    Sketch(SketchDef),
    Extrude(ExtrudeDef),
    Revolve(RevolveDef),
    Sweep(SweepDef),
    Compound,
    Assembly(AssemblyDef),
    Fillet(FilletDef),
    Chamfer(ChamferDef),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyDef {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilletDef {
    pub radius: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChamferDef {
    pub distance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxDef {
    pub width: f64,
    pub height: f64,
    pub depth: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CylinderDef {
    pub radius: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SphereDef {
    pub radius: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshDef {
    pub vertices: Vec<[f64; 3]>,
    pub indices: Vec<[u32; 3]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SketchDef {
    pub primitives: Vec<SketchPrimitive>,
    pub plane: SketchPlane,
    pub extrude: Option<ExtrudeDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SketchPlane {
    XY,
    XZ,
    YZ,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SketchPrimitive {
    Rectangle { x: f64, y: f64, width: f64, height: f64 },
    Circle { cx: f64, cy: f64, radius: f64, segments: Option<u32> },
    Polygon { points: Vec<[f64; 2]> },
    Arc { cx: f64, cy: f64, radius: f64, start_angle: f64, end_angle: f64, segments: Option<u32> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtrudeDef {
    pub sketch_id: String,
    pub distance: f64,
    pub direction: Option<[f64; 3]>,
    pub cap: bool,
    pub taper_angle: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevolveDef {
    pub sketch_id: String,
    pub angle: f64,
    pub segments: Option<u32>,
    pub axis: RevolveAxis,
    pub cap: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RevolveAxis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepDef {
    pub sketch_id: String,
    pub path: SweepPath,
    pub segments: Option<u32>,
    pub cap: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SweepPath {
    Linear { direction: [f64; 3], distance: f64 },
    Arc { radius: f64, angle: f64, axis: [f64; 3] },
    Helix { radius: f64, pitch: f64, turns: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformOp {
    pub translation: Option<[f64; 3]>,
    pub rotation: Option<[f64; 3]>,
    pub scale: Option<[f64; 3]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsgOperation {
    pub op_type: CsgOpType,
    pub tool_id: String,
    pub tool_transform: Option<TransformOp>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CsgOpType {
    Union,
    Subtract,
    Intersect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometryOutput {
    pub brep: BRepModel,
    pub mesh: TriangleMesh,
    pub world_matrices: HashMap<String, [f64; 16]>,
    pub outline_2d: Sketch2D,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriangleMesh {
    pub vertices: Vec<[f64; 3]>,
    pub normals: Vec<[f64; 3]>,
    pub uvs: Vec<[f64; 2]>,
    pub triangles: Vec<[u32; 3]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BRepModel {
    pub solids: Vec<Solid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Solid {
    pub id: String,
    pub faces: Vec<Face>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Face {
    pub id: String,
    pub source_solid: Option<String>,
    pub source_face: Option<String>,
    pub operation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sketch2D {
    pub contours: Vec<Vec<[f64; 2]>>,
}
