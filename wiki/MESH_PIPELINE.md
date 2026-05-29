# Mesh Pipeline

## Overview

The mesh pipeline transforms the scene tree (`GeometryNode` hierarchy + joints) into renderable Bevy meshes. It runs each frame to detect changes and update the GPU representation incrementally.

### Phases

```
Scene Tree → evaluate_all() → SceneGeometry → sync_meshes() → MeshCache → Bevy ECS
```

---

## 1. evaluate_all()

`apps/desktop/src/document.rs:60` — Called whenever the document is marked dirty. Walks the full scene tree to produce `SceneGeometry`:

```rust
pub struct SceneGeometry {
    pub meshes: HashMap<String, TriangleMesh>,
}
```

The process has three steps:

### 1a. compute_world_matrices()

`document.rs:171` — Recursively computes a `HashMap<String, DMat4>` of world-space transforms by traversing the tree depth-first:

```
world(child) = world(parent) * local_matrix(child)
```

`local_matrix()` (`document.rs:149`) builds a 4×4 matrix from a node's `TransformOp`: translation first, then ZYX rotation, then scale.

### 1b. Joint integration

`document.rs:225` — `build_mesh_with_joint_context()` checks whether a node is a child in a joint relationship. If so, the joint matrix from `JointEngine::compute_joint_matrix()` is inserted between the parent's world matrix and the child:

```
world = parent_world * joint_matrix
```

The resulting world matrix is applied directly to each vertex of the built mesh.

### 1c. Per-node hashing

`document.rs:108` — `hash_geometry_node()` computes a hash from the node's type-specific parameters (dimensions of Box, Cylinder, Sphere). Other node types return 0 (meaning "always re-evaluate"). During `collect_evaluated_meshes()` (`document.rs:193`), the new hash is compared to the old hash stored in `Document.node_hashes`. If the hash matches, the previous mesh is reused without rebuilding — a significant optimization for complex scenes.

### 1d. Mesh building

`document.rs:286` — `build_mesh_from_node_in_context()` collects all sketch definitions from the scene, then uses `MeshBuilder` (from `kpe-geometry`) to produce a `TriangleMesh` with vertices, normals, UVs, and triangle indices.

---

## 2. SceneGeometry

`document.rs:11` — The output of evaluation. A flat map from node ID to `TriangleMesh`. Container nodes (`Compound`, `Assembly`) produce no entries. Hidden nodes are not filtered here — that happens in `sync_meshes()`.

---

## 3. sync_meshes()

`apps/desktop/src/sync.rs:40` — A Bevy system that runs every frame. It bridges the data-oriented `SceneGeometry` to Bevy's ECS with meshes, materials, and entities.

### Change detection

`sync.rs:53` — Compares `state.mesh_gen` (incremented by `mark_dirty()`) against `cache.last_gen`. If unchanged, the system returns immediately.

### Mesh creation / update

For each evaluated mesh not in the cache:
1. Convert `TriangleMesh` to a Bevy `Mesh` via `kpe_mesh_to_bevy()` (`sync.rs:120`).
2. Create or reuse a `StandardMaterial` with the node's hex color.
3. Spawn a child entity under `SceneMeshRoot` with `Mesh3d`, `MeshMaterial3d`, and `MeshNodeId`.

For cached meshes that still exist: update the mesh data in-place via `meshes.insert(handle.id(), bevy_mesh)` and update material color if changed (`sync.rs:70-77`).

### Hidden node filtering

`sync.rs:61-63` — Nodes in `Document.hidden_nodes` are skipped entirely during iteration. They are then collected as "stale" and despawned:

```rust
let hidden = &state.document.hidden_nodes;
for (id, tri_mesh) in &state.document.evaluated.meshes {
    if hidden.contains(id) { continue; }
    known.insert(id.clone());
    // ... create or update mesh
}
```

### Stale mesh cleanup

`sync.rs:108-117` — Any handle in the cache whose ID is no longer in `known` (either deleted or hidden) is despawned recursively and removed from the cache maps.

---

## 4. MeshCache

`sync.rs:9` — A Bevy resource that persists GPU handles across frames:

```rust
pub struct MeshCache {
    pub handles: HashMap<String, Handle<Mesh>>,
    pub entities: HashMap<String, Entity>,
    pub materials: HashMap<String, Handle<StandardMaterial>>,
    pub last_gen: u64,
}
```

- **handles**: Maps node ID → Bevy mesh handle for in-place updates.
- **entities**: Maps node ID → spawned entity (for cleanup and ray-picking).
- **materials**: Maps node ID → material handle (for color updates).
- **last_gen**: The last processed `mesh_gen` value, used for change detection.

---

## 5. MeshNodeId

`sync.rs:28` — A marker component linking a Bevy entity back to its scene node:

```rust
#[derive(Component)]
pub struct MeshNodeId(pub String);
```

Used by:
- **Ray-picking** (`main.rs:143`): The viewport selection system iterates all entities with `MeshNodeId` and tests ray-AABB intersection. On hit, the stored string becomes the selection.
- **Fit-all** (`camera.rs:38`): Queries `Aabb` components on entities with `MeshNodeId` to compute the scene bounding box.

---

## 6. SceneMeshRoot

`sync.rs:24` — A singleton entity spawned at startup (via `setup_scene()` in `sync.rs:30`) that acts as the parent for all mesh entities. This keeps the hierarchy clean and allows despawning all children at once if needed.
