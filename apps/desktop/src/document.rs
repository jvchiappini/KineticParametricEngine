use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use kpe_geometry::mesh::MeshBuilder;
use kpe_schema::geometry::{BoxDef, CylinderDef, GeometryNode, GeometryNodeType, SphereDef, TriangleMesh, TransformOp};
use kpe_schema::recipe::KPERecipe;
use kpe_schema::joint::Joint;
use kpe_geometry::joint::JointEngine;
use glam::{DMat4, DVec3, DVec4};

#[derive(Debug, Clone)]
pub struct SceneGeometry {
    pub meshes: HashMap<String, TriangleMesh>,
}

impl SceneGeometry {
    pub fn new() -> Self {
        Self { meshes: HashMap::new() }
    }

    pub fn triangle_count(&self) -> usize {
        self.meshes.values().map(|m| m.triangles.len()).sum()
    }
}

impl Default for SceneGeometry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Document {
    pub recipe: KPERecipe,
    pub evaluated: SceneGeometry,
    pub selection: Option<String>,
    pub multi_selection: Vec<String>,
    pub joint_selection: Option<String>,
    pub file_path: Option<String>,
    pub is_modified: bool,
    pub hidden_nodes: HashSet<String>,
    node_hashes: HashMap<String, u64>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            recipe: KPERecipe::default(),
            evaluated: SceneGeometry::new(),
            selection: None,
            multi_selection: Vec::new(),
            joint_selection: None,
            file_path: None,
            is_modified: false,
            hidden_nodes: HashSet::new(),
            node_hashes: HashMap::new(),
        }
    }

    pub fn evaluate_all(&mut self) {
        let mut meshes = HashMap::new();
        let mut hashes = HashMap::new();
        let joints = &self.recipe.joints;
        let old_meshes = &self.evaluated.meshes;
        let full_scene = &self.recipe.scene;

        // Pre-compute world matrices for all nodes
        let world_matrices = compute_world_matrices(full_scene, DMat4::IDENTITY);

        // Compute all evaluated meshes with joint awareness
        collect_evaluated_meshes(
            full_scene,
            &world_matrices,
            joints,
            &mut meshes,
            &mut hashes,
            &self.node_hashes,
            old_meshes,
            full_scene,
        );
        self.evaluated = SceneGeometry { meshes };
        self.node_hashes = hashes;
    }

    pub fn evaluate_node(&mut self, node_id: &str) {
        if let Some(node) = find_node(&self.recipe.scene, node_id) {
            let new_hash = hash_geometry_node(node);
            let old_hash = self.node_hashes.get(node_id).copied().unwrap_or(0);
            if new_hash != 0 && new_hash == old_hash {
                return;
            }
            let world_matrices = compute_world_matrices(&self.recipe.scene, DMat4::IDENTITY);
            let mesh = build_mesh_with_joint_context(node, &self.recipe.scene, &world_matrices, &self.recipe.joints);
            self.evaluated.meshes.insert(node_id.to_string(), mesh);
            if new_hash != 0 {
                self.node_hashes.insert(node_id.to_string(), new_hash);
            }
        }
    }

    pub fn all_node_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();
        collect_ids(&self.recipe.scene, &mut ids);
        ids
    }
}

fn hash_geometry_node(node: &GeometryNode) -> u64 {
    let mut s = DefaultHasher::new();
    match &node.node_type {
        GeometryNodeType::Box(b) => {
            0u64.hash(&mut s);
            hash_box_def(b).hash(&mut s);
        }
        GeometryNodeType::Cylinder(c) => {
            1u64.hash(&mut s);
            hash_cylinder_def(c).hash(&mut s);
        }
        GeometryNodeType::Sphere(sp) => {
            2u64.hash(&mut s);
            hash_sphere_def(sp).hash(&mut s);
        }
        _ => return 0,
    }
    s.finish()
}

fn hash_box_def(b: &BoxDef) -> u64 {
    let mut s = DefaultHasher::new();
    b.width.to_bits().hash(&mut s);
    b.height.to_bits().hash(&mut s);
    b.depth.to_bits().hash(&mut s);
    s.finish()
}

fn hash_cylinder_def(c: &CylinderDef) -> u64 {
    let mut s = DefaultHasher::new();
    c.radius.to_bits().hash(&mut s);
    c.height.to_bits().hash(&mut s);
    s.finish()
}

fn hash_sphere_def(sp: &SphereDef) -> u64 {
    let mut s = DefaultHasher::new();
    sp.radius.to_bits().hash(&mut s);
    s.finish()
}

fn local_matrix(tf: &Option<TransformOp>) -> DMat4 {
    match tf {
        Some(t) => {
            let mut mat = DMat4::IDENTITY;
            if let Some(trans) = &t.translation {
                mat = DMat4::from_translation(DVec3::new(trans[0], trans[1], trans[2]));
            }
            if let Some(rot) = &t.rotation {
                let rx = DMat4::from_rotation_x(rot[0].to_radians());
                let ry = DMat4::from_rotation_y(rot[1].to_radians());
                let rz = DMat4::from_rotation_z(rot[2].to_radians());
                mat = mat * rz * ry * rx;
            }
            if let Some(scale) = &t.scale {
                mat = mat * DMat4::from_scale(DVec3::new(scale[0], scale[1], scale[2]));
            }
            mat
        }
        None => DMat4::IDENTITY,
    }
}

fn compute_world_matrices(node: &GeometryNode, parent_world: DMat4) -> HashMap<String, DMat4> {
    let mut map = HashMap::new();
    let local = local_matrix(&node.transform);
    let world = parent_world * local;
    map.insert(node.id.clone(), world);
    for child in &node.children {
        let child_maps = compute_world_matrices(child, world);
        map.extend(child_maps);
    }
    map
}

fn find_joint_parent_world(
    world_matrices: &HashMap<String, DMat4>,
    joints: &[Joint],
    child_id: &str,
) -> Option<DMat4> {
    joints.iter()
        .find(|j| j.child_id == child_id)
        .and_then(|j| world_matrices.get(&j.parent_id).copied())
}

fn collect_evaluated_meshes(
    node: &GeometryNode,
    world_matrices: &HashMap<String, DMat4>,
    joints: &[Joint],
    meshes: &mut HashMap<String, TriangleMesh>,
    hashes: &mut HashMap<String, u64>,
    old_hashes: &HashMap<String, u64>,
    old_meshes: &HashMap<String, TriangleMesh>,
    full_scene: &GeometryNode,
) {
    // Skip container types for individual evaluation
    let is_container = matches!(node.node_type, GeometryNodeType::Compound | GeometryNodeType::Assembly(_));
    if !is_container {
        let new_hash = hash_geometry_node(node);
        let old_hash = old_hashes.get(&node.id).copied().unwrap_or(0);
        if new_hash != 0 && new_hash == old_hash {
            if let Some(old_mesh) = old_meshes.get(&node.id) {
                meshes.insert(node.id.clone(), old_mesh.clone());
            }
        } else {
            let mesh = build_mesh_with_joint_context(node, full_scene, world_matrices, joints);
            meshes.insert(node.id.clone(), mesh);
        }
        if new_hash != 0 {
            hashes.insert(node.id.clone(), new_hash);
        }
    }
    for child in &node.children {
        collect_evaluated_meshes(child, world_matrices, joints, meshes, hashes, old_hashes, old_meshes, full_scene);
    }
}

fn build_mesh_with_joint_context(
    node: &GeometryNode,
    full_scene: &GeometryNode,
    world_matrices: &HashMap<String, DMat4>,
    joints: &[Joint],
) -> TriangleMesh {
    // Check if this node is a child in a joint
    if let Some(joint) = joints.iter().find(|j| j.child_id == node.id) {
        if let Some(parent_world) = world_matrices.get(&joint.parent_id) {
            let engine = JointEngine::new();
            let joint_matrix = engine.compute_joint_matrix(joint);
            // mesh built by build_from_node already has local transform baked in,
            // so we only need parent_world * joint_matrix on top
            let world = *parent_world * joint_matrix;

            let mut sketches = HashMap::new();
            kpe_geometry::mesh::collect_sketches(full_scene, &mut sketches);
            let builder = MeshBuilder::new().with_sketches(sketches);
            let mut mesh = builder.build_from_node(node);

            // Override world transform: apply the joint-aware world
            if world != DMat4::IDENTITY {
                for v in &mut mesh.vertices {
                    let p = world * DVec4::new(v[0], v[1], v[2], 1.0);
                    v[0] = p.x / p.w;
                    v[1] = p.y / p.w;
                    v[2] = p.z / p.w;
                }
            }
            return mesh;
        }
    }
    // No joint: normal evaluation
    build_mesh_from_node_in_context(node, full_scene)
}

fn find_node<'a>(node: &'a GeometryNode, target_id: &str) -> Option<&'a GeometryNode> {
    if node.id == target_id {
        return Some(node);
    }
    for child in &node.children {
        if let result @ Some(_) = find_node(child, target_id) {
            return result;
        }
    }
    None
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

fn collect_ids(node: &GeometryNode, ids: &mut Vec<String>) {
    ids.push(node.id.clone());
    for child in &node.children {
        collect_ids(child, ids);
    }
}

fn build_mesh_from_node_in_context(node: &GeometryNode, full_scene: &GeometryNode) -> TriangleMesh {
    let mut sketches = std::collections::HashMap::new();
    kpe_geometry::mesh::collect_sketches(full_scene, &mut sketches);
    let builder = MeshBuilder::new().with_sketches(sketches);
    builder.build_from_node(node)
}

pub fn build_mesh_with_joints(node: &GeometryNode, joints: &[Joint]) -> TriangleMesh {
    let mut sketches = std::collections::HashMap::new();
    kpe_geometry::mesh::collect_sketches(node, &mut sketches);
    let builder = MeshBuilder::new().with_sketches(sketches).with_joints(joints.to_vec());
    builder.build_from_node(node)
}
