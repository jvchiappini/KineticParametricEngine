# Data Flow — End to End

```
User edits parameters in UI (React/TS)
  │
  ▼
JSON KPERecipe
  │
  ├──► kpe-parametric::Solver
  │     │
  │     ├── expression::evaluate()
  │     │   Parses "params.width - 2 * params.thickness"
  │     │   → resolves variable dependency graph
  │     │
  │     ├── condition::evaluate()
  │     │   Evaluates "params.width > 800" → true/false
  │     │
  │     ├── rule_engine::apply()
  │     │   If condition met:
  │     │     • add_child → insert GeometryNode into scene tree
  │     │     • add_operation → insert CsgOperation
  │     │     • set_param → override a parameter
  │     │
  │     └── Produces: ResolvedRecipe
  │           (all params resolved to concrete numbers,
  │            rules applied, scene tree expanded)
  │
  ▼
kpe-geometry
  │
  ├── mesh::MeshBuilder
  │     BoxDef → 8 vertices + 12 triangles
  │     CylinderDef → 2N vertices + 2N triangles
  │     SphereDef → rings×segments vertices + 2×rings×segments triangles
  │
  ├── transform::TransformEngine
  │     Computes Mat4 from TransformOp
  │     Accumulates world matrix: parent_world × local
  │     Matrices are NOT baked into vertices (per ADR-004)
  │
  ├── csg::CsgKernel  (per ADR-005)
  │     │
  │     ├── bvh::BVH
  │     │     Builds AABB tree for each input mesh
  │     │     Queries intersecting triangle pairs O(n log n)
  │     │
  │     ├── intersection::triangle_triangle_intersection()
  │     │     Möller 1997: returns None / Coplanar / Segment
  │     │
  │     ├── classify::classify_triangle_fragments()
  │     │     Ray-casting winding number for each triangle centroid
  │     │
  │     ├── stitch::Stitcher
  │     │     Welds vertices within epsilon
  │     │     Removes degenerate/duplicate triangles
  │     │
  │     └── Output per operation:
  │           union: A_outside_B ∪ B_outside_A
  │           subtract: A_outside_B ∪ B_inside_A (flipped normals)
  │           intersect: A_inside_B ∪ B_inside_A
  │
  ├── joint::JointEngine
  │     Revolute: to_pivot × rotate(angle, axis) × from_pivot
  │     Prismatic: translate(axis × value)
  │     Clamp to JointLimits { min, max }
  │
  └── Produces: GeometryOutput
        {
          mesh: TriangleMesh,        → vertices + triangles for renderer
          brep: BRepModel,           → half-edge skeleton (future)
          world_matrices: [f64;16],  → per node, for renderer
          outline_2d: Sketch2D       → for DXF / SVG
        }
  │
  ├──► kpe-fabrication
  │     ├── cutlist::generate()
  │     │     Walks scene tree, extracts CutPiece for each box
  │     │
  │     ├── nesting::optimize()
  │     │     Guillotine bin-packing on sheet dimensions
  │     │     Respects grain direction constraints
  │     │
  │     ├── dxf::export()
  │     │     LWPOLYLINE entities per nested piece
  │     │
  │     └── svg::export()
  │           <rect> elements per nested piece
  │
  └──► External Renderer (your custom renderer / Three.js / wgpu)
        Receives TriangleMesh + world_matrices
        Applies transforms via GPU (not CPU)
        Draws at 60fps — joint changes only update matrices
```

## Key Architectural Properties

```
1. Renderer never modifies the model
   ────────────────────────────────
   State lives entirely in KPERecipe.
   Renderer consumes GeometryOutput read-only.

2. Parametric change → full re-resolve
   ────────────────────────────────────
   Parameter change → Solver::resolve() → GeometryBuilder → new GeometryOutput
   This is intentional: the pipeline is fast enough (milliseconds for
   typical furniture models), and it guarantees consistency.

3. Joint movement → matrix-only update
   ────────────────────────────────────
   Joint::current_value changes → only world_matrices recomputed.
   No geometry rebuild, no CSG re-execution.
   This is the path for 60fps interactive manipulation.
```

## Crate Dependency Graph

```
   kpe-schema     (shared types, no deps on other kpe crates)
       ↑
       ├─── kpe-parametric    (solver, expressions, rules)
       ├─── kpe-geometry      (meshes, CSG, transforms, joints)
       ├─── kpe-fabrication   (cut lists, nesting, DXF/SVG)
       ├─── kpe-material      (procedural textures, UVs)
       └─── kpe-wasm          (wasm-bindgen bindings)
                                  ↑
                            (consumed by apps/)
```
