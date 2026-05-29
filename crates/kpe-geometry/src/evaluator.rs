use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use glam::{DMat4, DVec4};
use kpe_schema::geometry::{
    BoxDef, CylinderDef, GeometryNode, GeometryNodeType, SphereDef, TriangleMesh, TransformOp,
};
use kpe_schema::joint::Joint;
use kpe_schema::recipe::KPERecipe;

use crate::joint::JointEngine;
use crate::mesh::MeshBuilder;

/// The result of evaluating a complete scene.
#[derive(Debug, Clone)]
pub struct SceneGeometry {
    pub meshes: HashMap<String, TriangleMesh>,
}

impl SceneGeometry {
    pub fn new() -> Self {
        Self {
            meshes: HashMap::new(),
        }
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

// ── Master public API ─────────────────────────────────────────────

/// Evaluate a full `KPERecipe` and produce the geometry for every node.
///
/// `old_hashes` and `old_meshes` are optional caches from a previous
/// evaluation.  When a node's hash matches, the old mesh is reused.
pub fn evaluate_scene(
    recipe: &KPERecipe,
    old_hashes: &HashMap<String, u64>,
    old_meshes: &HashMap<String, TriangleMesh>,
) -> SceneGeometry {
    let mut meshes = HashMap::new();
    let mut _new_hashes = HashMap::new();
    let joints = &recipe.joints;
    let full_scene = &recipe.scene;

    let world_matrices = compute_world_matrices(full_scene, DMat4::IDENTITY);

    collect_evaluated_meshes(
        full_scene,
        &world_matrices,
        joints,
        &mut meshes,
        &mut _new_hashes,
        old_hashes,
        old_meshes,
        full_scene,
    );

    SceneGeometry { meshes }
}

/// Evaluate a single node by ID within the full recipe context.
pub fn evaluate_node(
    node_id: &str,
    recipe: &KPERecipe,
    old_hashes: &HashMap<String, u64>,
) -> Option<TriangleMesh> {
    let node = find_node(&recipe.scene, node_id)?;
    let new_hash = hash_geometry_node(node);
    let old_hash = old_hashes.get(node_id).copied().unwrap_or(0);
    if new_hash != 0 && new_hash == old_hash {
        return None; // unchanged — caller should keep old mesh
    }
    let world_matrices = compute_world_matrices(&recipe.scene, DMat4::IDENTITY);
    let mesh = build_mesh_with_joint_context(node, &recipe.scene, &world_matrices, &recipe.joints);
    Some(mesh)
}

// ── World matrix computation ──────────────────────────────────────

/// Compute the local transform matrix from an optional `TransformOp`.
///
/// This is identical to `mesh::local_matrix` but replicated here to keep
/// the evaluator self-contained and avoid growing `mesh.rs` further.
fn local_matrix(tf: &Option<TransformOp>) -> DMat4 {
    match tf {
        Some(t) => {
            let mut mat = DMat4::IDENTITY;

            if let Some(trans) = &t.translation {
                mat = DMat4::from_translation(glam::DVec3::new(trans[0], trans[1], trans[2]));
            }

            if let Some(rot) = &t.rotation {
                let rx = DMat4::from_rotation_x(rot[0].to_radians());
                let ry = DMat4::from_rotation_y(rot[1].to_radians());
                let rz = DMat4::from_rotation_z(rot[2].to_radians());
                mat = mat * rz * ry * rx;
            }

            if let Some(scale) = &t.scale {
                mat = mat * DMat4::from_scale(glam::DVec3::new(scale[0], scale[1], scale[2]));
            }

            mat
        }
        None => DMat4::IDENTITY,
    }
}

/// Recursively compute the world-space transform for every node.
pub fn compute_world_matrices(node: &GeometryNode, parent_world: DMat4) -> HashMap<String, DMat4> {
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

// ── Hashing helpers ───────────────────────────────────────────────

/// Compute a content hash for a geometry node.
///
/// Returns 0 for node types that cannot be meaningfully hashed
/// (compound, extrude, etc.).
pub fn hash_geometry_node(node: &GeometryNode) -> u64 {
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

// ── Recursive mesh collection ─────────────────────────────────────

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
    let is_container =
        matches!(node.node_type, GeometryNodeType::Compound | GeometryNodeType::Assembly(_));

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
        collect_evaluated_meshes(
            child,
            world_matrices,
            joints,
            meshes,
            hashes,
            old_hashes,
            old_meshes,
            full_scene,
        );
    }
}

// ── Joint-aware mesh building ─────────────────────────────────────

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
            crate::mesh::collect_sketches(full_scene, &mut sketches);
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

fn build_mesh_from_node_in_context(node: &GeometryNode, full_scene: &GeometryNode) -> TriangleMesh {
    let mut sketches = HashMap::new();
    crate::mesh::collect_sketches(full_scene, &mut sketches);
    let builder = MeshBuilder::new().with_sketches(sketches);
    builder.build_from_node(node)
}

/// Build a mesh for a single node with joint context (no scene tree needed).
pub fn build_mesh_with_joints(node: &GeometryNode, joints: &[Joint]) -> TriangleMesh {
    let mut sketches = HashMap::new();
    crate::mesh::collect_sketches(node, &mut sketches);
    let builder = MeshBuilder::new().with_sketches(sketches).with_joints(joints.to_vec());
    builder.build_from_node(node)
}

// ── Tree traversal utilities ──────────────────────────────────────

/// Find a node by ID in the scene tree.
pub fn find_node<'a>(node: &'a GeometryNode, target_id: &str) -> Option<&'a GeometryNode> {
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

/// Find the parent of a node by the child's ID.
pub fn find_parent<'a>(node: &'a GeometryNode, target: &str) -> Option<&'a GeometryNode> {
    for child in &node.children {
        if child.id == target {
            return Some(node);
        }
        if let found @ Some(_) = find_parent(child, target) {
            return found;
        }
    }
    None
}

/// Mutable version of `find_parent`.
pub fn find_parent_mut<'a>(
    node: &'a mut GeometryNode,
    target: &str,
) -> Option<&'a mut GeometryNode> {
    for child in &mut node.children {
        if child.id == target {
            return Some(node);
        }
    }
    for child in &mut node.children {
        if let found @ Some(_) = find_parent_mut(child, target) {
            return found;
        }
    }
    None
}

/// Collect all node IDs from the tree.
pub fn collect_ids(node: &GeometryNode, ids: &mut Vec<String>) {
    ids.push(node.id.clone());
    for child in &node.children {
        collect_ids(child, ids);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kpe_schema::geometry::BoxDef;

    #[test]
    fn test_evaluate_scene_empty() {
        let recipe = KPERecipe::default();
        let result = evaluate_scene(&recipe, &HashMap::new(), &HashMap::new());
        assert!(result.meshes.is_empty());
    }

    #[test]
    fn test_compute_world_matrices_root() {
        let node = GeometryNode {
            id: "root".into(),
            node_type: GeometryNodeType::Compound,
            transform: None,
            children: vec![],
            operations: vec![],
            color: None,
        };
        let mats = compute_world_matrices(&node, DMat4::IDENTITY);
        assert!(mats.contains_key("root"));
        assert_eq!(mats.len(), 1);
    }

    #[test]
    fn test_hash_box() {
        let node = GeometryNode {
            id: "b1".into(),
            node_type: GeometryNodeType::Box(BoxDef {
                width: 2.0,
                height: 3.0,
                depth: 4.0,
            }),
            transform: None,
            children: vec![],
            operations: vec![],
            color: None,
        };
        let h = hash_geometry_node(&node);
        assert_ne!(h, 0);
        // Same box → same hash
        let node2 = GeometryNode { ..node.clone() };
        assert_eq!(h, hash_geometry_node(&node2));
    }

    #[test]
    fn test_find_node() {
        let child = GeometryNode {
            id: "child".into(),
            node_type: GeometryNodeType::Compound,
            transform: None,
            children: vec![],
            operations: vec![],
            color: None,
        };
        let parent = GeometryNode {
            id: "parent".into(),
            node_type: GeometryNodeType::Compound,
            transform: None,
            children: vec![child],
            operations: vec![],
            color: None,
        };
        assert!(find_node(&parent, "child").is_some());
        assert!(find_node(&parent, "nonexistent").is_none());
    }

    #[test]
    fn test_find_parent() {
        let child = GeometryNode {
            id: "child".into(),
            node_type: GeometryNodeType::Compound,
            transform: None,
            children: vec![],
            operations: vec![],
            color: None,
        };
        let parent = GeometryNode {
            id: "parent".into(),
            node_type: GeometryNodeType::Compound,
            transform: None,
            children: vec![child],
            operations: vec![],
            color: None,
        };
        let found = find_parent(&parent, "child");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "parent");
    }
}
