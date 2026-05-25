use std::collections::HashMap;
use glam::DVec3;

pub type VertexId = u64;
pub type HalfEdgeId = u64;
pub type FaceId = u64;
pub type SolidId = u64;

#[derive(Debug, Clone)]
pub struct Vertex {
    pub id: VertexId,
    pub position: DVec3,
}

#[derive(Debug, Clone)]
pub struct HalfEdge {
    pub id: HalfEdgeId,
    pub vertex: VertexId,
    pub face: FaceId,
    pub twin: HalfEdgeId,
    pub next: HalfEdgeId,
}

#[derive(Debug, Clone)]
pub struct FaceMetadata {
    pub source_solid: Option<SolidId>,
    pub source_face: Option<FaceId>,
    pub operation: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Face {
    pub id: FaceId,
    pub half_edge: HalfEdgeId,
    pub metadata: FaceMetadata,
}

#[derive(Debug, Clone)]
pub struct Solid {
    pub id: SolidId,
    pub faces: Vec<FaceId>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BRepKernel {
    vertices: HashMap<VertexId, Vertex>,
    half_edges: HashMap<HalfEdgeId, HalfEdge>,
    faces: HashMap<FaceId, Face>,
    solids: HashMap<SolidId, Solid>,
    next_vertex_id: VertexId,
    next_half_edge_id: HalfEdgeId,
    next_face_id: FaceId,
    next_solid_id: SolidId,
}

impl BRepKernel {
    pub fn new() -> Self {
        Self {
            vertices: HashMap::new(),
            half_edges: HashMap::new(),
            faces: HashMap::new(),
            solids: HashMap::new(),
            next_vertex_id: 1,
            next_half_edge_id: 1,
            next_face_id: 1,
            next_solid_id: 1,
        }
    }

    pub fn add_vertex(&mut self, position: DVec3) -> VertexId {
        let id = self.next_vertex_id;
        self.next_vertex_id += 1;
        self.vertices.insert(id, Vertex { id, position });
        id
    }

    pub fn get_vertex(&self, id: VertexId) -> Option<&Vertex> {
        self.vertices.get(&id)
    }

    pub fn add_face(&mut self, half_edge: HalfEdgeId, solid_id: Option<SolidId>) -> FaceId {
        let id = self.next_face_id;
        self.next_face_id += 1;
        self.faces.insert(id, Face {
            id,
            half_edge,
            metadata: FaceMetadata {
                source_solid: solid_id,
                source_face: None,
                operation: None,
            },
        });
        id
    }

    pub fn get_face(&self, id: FaceId) -> Option<&Face> {
        self.faces.get(&id)
    }

    pub fn add_solid(&mut self, face_ids: Vec<FaceId>) -> SolidId {
        let id = self.next_solid_id;
        self.next_solid_id += 1;
        self.solids.insert(id, Solid {
            id,
            faces: face_ids,
        });
        id
    }

    pub fn get_solid(&self, id: SolidId) -> Option<&Solid> {
        self.solids.get(&id)
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    pub fn solid_count(&self) -> usize {
        self.solids.len()
    }
}

impl Default for BRepKernel {
    fn default() -> Self {
        Self::new()
    }
}
