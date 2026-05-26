# Data Flow — End to End

```
User edits parameters in UI / loads recipe JSON (apps/web or apps/cli)
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
kpe-geometry::MeshBuilder
  │
  ├── Primitives
  │     BoxDef → 8 vertices + 12 triangles
  │     CylinderDef → 2N vertices + 2N triangles
  │     SphereDef → rings×segments vertices + 2×rings×segments triangles
  │
  ├── Sketch Pipeline (per ADR-006)
  │     │
  │     ├── sketch::tessellate_sketch()
  │     │     Rectangle → 4 vertices (closed polyline)
  │     │     Circle → N vertices (N = segments, default 32)
  │     │     Polygon → M vertices (M = point count)
  │     │     Arc → K vertices (K = segments, default 16)
  │     │
  │     └── extrude::extrude_sketch()
  │           For each contour:
  │           • Project 2D → 3D via SketchPlane (XY / XZ / YZ)
  │           • Bottom cap: fan from first vertex (clockwise)
  │           • Top cap: fan from first vertex (counter-clockwise)
  │           • Side walls: quad strips (2 triangles per segment)
  │           → TriangleMesh
  │
  ├── Compound nodes
  │     Recursively build children, merge meshes with offset indices
  │
  ├── CSG Operations (csgrs backend)
  │     │
  │     ├── triangle_mesh_to_csg()
  │     │     Each triangle → Polygon with 3 Vertex (pos + normal)
  │     │     → csgrs::Mesh<()>
  │     │
  │     ├── Boolean operation via csgrs
  │     │     • union()        → A ∪ B
  │     │     • difference()   → A \ B
  │     │     • intersection() → A ∩ B
  │     │     csgrs uses BSP-tree with exact arithmetic
  │     │
  │     └── csg_to_triangle_mesh()
  │           Triangulate output polygons
  │           Deduplicate vertices via quantised hash (1e-5)
  │           → TriangleMesh (with computed normals)
  │
  ├── Transform Engine (per ADR-004)
  │     Computes Mat4 from TransformOp
  │     Accumulates world matrix: parent_world × local
  │     Matrices are NOT baked into vertices
  │
  ├── Joint Engine
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
  │     └── DXF / SVG export
  │           LWPOLYLINE, 3DFACE, <rect> elements
  │
  ├──► kpe-cli (export)
  │     ├── kpe export recipe.json output.step
  │     │     STEP AP242 tessellation (TRIANGULATED_FACE)
  │     │
  │     └── kpe export recipe.json output.dxf
  │           DXF AC1009, 3DFACE entities per triangle
  │
  └──► apps/web (Three.js + WASM)
        init() → loads WASM
        build_mesh() / csg_union() / csg_subtract() / csg_intersect()
        → TriangleMesh JSON → Three.js BufferGeometry
        OrbitControls for navigation
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
       ├─── kpe-geometry      (meshes, CSG via csgrs, sketch, extrude, transforms, joints)
       ├─── kpe-fabrication   (cut lists, nesting, DXF)
       ├─── kpe-material      (procedural textures, UVs)
       └─── kpe-wasm          (wasm-bindgen bindings)
                                  ↑
                            (consumed by apps/)
```
