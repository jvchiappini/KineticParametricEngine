# Data Model

## GeometryNode

`crates/kpe-schema/src/geometry.rs:4` — The fundamental building block of the scene graph. Every visible or organizational element in the model is a `GeometryNode`. Nodes form a tree where each node holds an optional local transform, a list of child nodes, CSG operations, and an optional display color.

```rust
pub struct GeometryNode {
    pub id: String,
    pub node_type: GeometryNodeType,
    pub transform: Option<TransformOp>,
    pub children: Vec<GeometryNode>,
    pub operations: Vec<CsgOperation>,
    pub color: Option<String>,
}
```

- **id**: A unique string identifier (e.g. `"Box_001"`, `"root"`). Used for selection, joint references, and node lookup.
- **transform**: Optional local transformation (translation, rotation, scale) applied to the node's mesh *before* the parent-world chain is computed.
- **children**: Sub-nodes that inherit this node's world transform. The entire tree is rooted at a single Compound node called `"root"`.
- **operations**: CSG boolean operations (Union, Subtract, Intersect) that combine this node's mesh with sibling tool meshes. Defined at `geometry.rs:137`.
- **color**: Optional hex color string (e.g. `"#FF8800"`). Displayed in the 3D viewport; falls back to a default blue if `None`.

## GeometryNodeType

`geometry.rs:14` — An enum with one variant per primitive or feature type. Each variant carries its own definition struct.

| Variant | Definition Struct | Purpose |
|---|---|---|
| `Box(BoxDef)` | `BoxDef { width, height, depth }` | Rectangular prism with configurable dimensions (`geometry.rs:45`). |
| `Cylinder(CylinderDef)` | `CylinderDef { radius, height }` | Cylinder primitive (`geometry.rs:52`). |
| `Sphere(SphereDef)` | `SphereDef { radius }` | Sphere primitive (`geometry.rs:58`). |
| `Mesh(MeshDef)` | `MeshDef { vertices, indices }` | Raw imported mesh data (`geometry.rs:63`). |
| `Sketch(SketchDef)` | `SketchDef { primitives, plane }` | 2D sketch on a construction plane (`geometry.rs:69`). Contains a list of `SketchPrimitive` (Rectangle, Circle, Polygon, Arc) and a plane enum (`XY`, `XZ`, `YZ`). |
| `Extrude(ExtrudeDef)` | `ExtrudeDef { sketch_id, distance, direction, cap, taper_angle }` | Linear extrusion of a referenced sketch (`geometry.rs:90`). |
| `Revolve(RevolveDef)` | `RevolveDef { sketch_id, angle, segments, axis, cap }` | Revolved solid from a sketch around X, Y, or Z axis (`geometry.rs:99`). |
| `Sweep(SweepDef)` | `SweepDef { sketch_id, path, segments, cap }` | Swept solid along a path (Linear, Arc, or Helix) (`geometry.rs:115`). |
| `Compound` | (no data) | Container node — groups children without adding geometry. The root node is always `Compound`. |
| `Assembly(AssemblyDef)` | `AssemblyDef { name }` | Named grouping node for organizing sub-assemblies (`geometry.rs:30`). |
| `Fillet(FilletDef)` | `FilletDef { radius }` | Rounding operation applied to its child node (`geometry.rs:35`). |
| `Chamfer(ChamferDef)` | `ChamferDef { distance }` | Bevel operation applied to its child node (`geometry.rs:40`). |

Container types (`Compound`, `Assembly`) are skipped during individual mesh evaluation — their purpose is structural only.

## Joint

`crates/kpe-schema/src/joint.rs:3` — Represents a kinematic relationship between two nodes in the scene tree.

```rust
pub struct Joint {
    pub id: String,
    pub joint_type: JointType,
    pub parent_id: String,
    pub child_id: String,
    pub pivot: [f64; 3],
    pub axis: [f64; 3],
    pub limits: Option<JointLimits>,
    pub current_value: f64,
}
```

- **parent_id / child_id**: The two nodes being connected. The joint's transformation matrix is applied to the child relative to the parent's world matrix.
- **pivot**: Local pivot point offset for the joint.
- **axis**: Direction vector of the joint's degree of freedom.
- **current_value**: The current parameter value (angle in degrees for Revolute/Ball, distance for Prismatic).
- **limits**: Optional `JointLimits { min, max, damping, stiffness }`.

### JointType

`joint.rs:15` — Four kinematic types:

| Variant | Degrees of Freedom | Parameter Meaning |
|---|---|---|
| `Revolute` | 1 (rotation) | Angle around axis. Most common for hinges. |
| `Prismatic` | 1 (translation) | Linear distance along axis. |
| `Fixed` | 0 | Rigidly attaches child to parent. |
| `Ball` | 3 (rotation) | Spherical joint — angle around each axis. |

The `JointEngine` (in `kpe-geometry`) computes a 4×4 joint matrix from the joint definition, which is then composed with the parent's world matrix when evaluating the child mesh.

## KPERecipe

`crates/kpe-schema/src/recipe.rs:9` — The top-level document format, serialized as JSON via serde.

```rust
pub struct KPERecipe {
    pub version: String,
    pub metadata: RecipeMetadata,
    pub blocks: HashMap<String, BlockDefinition>,
    pub scene: GeometryNode,
    pub joints: Vec<Joint>,
    pub constraints: Vec<Constraint>,
    pub materials: HashMap<String, ProceduralMaterial>,
    pub precision: Option<f64>,
}
```

- **version**: Schema version string (`"0.1.0"` by default).
- **metadata**: `RecipeMetadata { name, author, description, created_at, tags }` — document provenance.
- **blocks**: Reusable parameter blocks driving dimensions via rules and variables. Not yet integrated into the UI.
- **scene**: The root `GeometryNode` — always `Compound` — that roots the entire scene tree.
- **joints**: All kinematic joints in the document.
- **constraints**: Assembly-level constraints (distinct from sketch constraints).
- **materials**: Named procedural materials keyed by ID.
- **precision**: Numerical tolerance for geometry operations.

## TriangleMesh

`geometry.rs:159` — The output representation of evaluated geometry:

```rust
pub struct TriangleMesh {
    pub vertices: Vec<[f64; 3]>,
    pub normals: Vec<[f64; 3]>,
    pub uvs: Vec<[f64; 2]>,
    pub triangles: Vec<[u32; 3]>,
}
```

Each triangle references three vertex indices. Normals and UVs are parallel arrays indexed the same as vertices. This is the format passed to Bevy for GPU rendering.

## TransformOp

`geometry.rs:130` — Optional local transformation:

```rust
pub struct TransformOp {
    pub translation: Option<[f64; 3]>,
    pub rotation: Option<[f64; 3]>,
    pub scale: Option<[f64; 3]>,
}
```

Rotation is stored in degrees (Euler angles, ZYX order). The local matrix is built as `Translate * RotZ * RotY * RotX * Scale` (see `document.rs:149`).

## Scene Tree

The scene tree is rooted at a `Compound` node with id `"root"`. Every other node is a child (direct or transitive) of root. The tree structure means child nodes inherit their parent's accumulated world transform. Container nodes (`Compound`, `Assembly`) have no mesh content themselves but organize children. `Fillet` and `Chamfer` nodes wrap exactly one child that they modify.

Operations (CSG) are stored per-node and combine the node's own mesh with sibling meshes referenced by `tool_id`. No feature tree / construction history is maintained — the scene tree *is* the only representation.

Node IDs are generated with zero-padded numeric suffixes (e.g. `Box_001`, `Cylinder_002`) by `feature_commands.rs:383` to avoid collisions.
