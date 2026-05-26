# KPE Architecture — Diagrams

## Layer Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                          APPLICATIONS                           │
│                                                                 │
│   apps/web (Vite + Three.js + WASM)        apps/cli (kpe-cli)   │
│   ┌─────────────────────┐            ┌─────────────────────┐    │
│   │  Three.js viewer    │            │  kpe export .step   │    │
│   │  CSG demo           │            │  kpe export .dxf    │    │
│   └──────────┬──────────┘            └──────────┬──────────┘    │
│              │ WASM                              │ Native        │
└──────────────┼──────────────────────────────────┼───────────────┘
               │                                  │
┌──────────────┼──────────────────────────────────┼───────────────┐
│              ▼            kpe-wasm              ▼               │
│  ┌─────────────────────────────────────────────────────┐        │
│  │           WASM Bindings (wasm-bindgen)              │        │
│  │  resolve_recipe, build_mesh, csg_union/subtract/    │        │
│  │  intersect — JSON in, JSON out                      │        │
│  └──────────────────────┬──────────────────────────────┘        │
└─────────────────────────┼───────────────────────────────────────┘
                          │
┌─────────────────────────┼───────────────────────────────────────┐
│                         ▼         CORE (Pure Rust)              │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                      kpe-schema                            │   │
│  │   KPERecipe, GeometryNode, TriangleMesh, SketchDef,       │   │
│  │   ExtrudeDef, CsgOperation, Joint, Constraint, Material   │   │
│  └──────────────────────────────────────────────────────────┘   │
│             ↑               ↑              ↑             ↑      │
│  ┌──────────┴──┐   ┌────────┴───┐   ┌──────┴──────┐  ┌──┴───┐  │
│  │kpe-parametric│   │kpe-geometry│   │kpe-fabricat.│  │kpe-mat.│  │
│  │              │   │            │   │             │  │       │  │
│  │ • solver     │   │ • csg(csgrs│   │ • cutlist   │  │ • gen │  │
│  │ • expression │   │   backend) │   │ • nesting   │  │ • uv  │  │
│  │ • condition  │   │ • sketch   │   │ • grain     │  │ • inst│  │
│  │ • rule_engine│   │ • extrude  │   │ • dxf/svg   │  │ • text│  │
│  │              │   │ • mesh     │   │             │  │       │  │
│  │              │   │ • stitches │   │             │  │       │  │
│  │              │   │ • bvh      │   │             │  │       │  │
│  │              │   │ • brep(ext)│   │             │  │       │  │
│  └──────────────┘   └──────┬─────┘   └─────────────┘  └───────┘  │
│                            │                                     │
└────────────────────────────┼─────────────────────────────────────┘
                             │
```

## CSG Pipeline (csgrs backend)

```
TriangleMesh A ─┐
                 ├──► triangle_mesh_to_csg()
TriangleMesh B ─┘         │
                          ▼
              csgrs::Mesh<()>
              (polygons with shared vertices)
                          │
                          ▼
              csgrs CSG boolean:
                • .union()
                • .difference()
                • .intersection()
                          │
                          ▼
              csg_to_triangle_mesh()
              (triangulate polygons, deduplicate vertices)
                          │
                          ▼
              TriangleMesh (output)
```

The conversion is O(N+M) for meshes of N and M triangles. `csgrs` uses a
BSP-tree-based boolean with exact arithmetic via `nalgebra` points. The
triangle-mesh round-trip adds vertex deduplication via quantised hash map
(1e-5 resolution) to heal cracks.

## Sketch + Extrude Pipeline

```
KPERecipe (JSON)
  scene:
    ├── node (id: "profile", type: Sketch)
    │     └── primitives: [Rectangle, Circle, Arc, ...]
    │         plane: XY | XZ | YZ
    │
    └── node (id: "part", type: Extrude)
          └── sketch_id: "profile"
              distance: 50.0
              cap: true
                          │
                          ▼
              tessellate_sketch()
              (primitives → Vec<Vec<DVec2>>)
                          │
                          ▼
              extrude_sketch()
              (contours → TriangleMesh:
               bottom cap, top cap, side walls)
                          │
                          ▼
              MeshBuilder.build_from_node()
              (merged into scene tree, passes to CSG)
```

## Primary Data Flow

```
User edits parameters in UI / loads recipe JSON
               │
               ▼
         KPERecipe (JSON)
         ┌─────────────────────────────────────┐
         │ version, metadata, blocks          │
         │ scene (GeometryNode tree)           │
         │ joints, constraints, materials      │
         └─────────────────────────────────────┘
               │
               ▼
     kpe-parametric::Solver
     ├── 1. Evaluate expressions: variables, arithmetic, floor
     ├── 2. Evaluate conditions: equals, not_equals, greater_than, etc.
     ├── 3. Apply rules: activate/deactivate geometry branches
     └── Produces: ResolvedRecipe (all concrete values)
               │
               ▼
     kpe-geometry::MeshBuilder
     ├── 1. Recursively build GeometryNode tree
     │    • Primitives: Box, Cylinder, Sphere → TriangleMesh
     │    • Sketch → tessellate (2D contours)
     │    • Extrude → extrude (contours → 3D mesh)
     │    • Compound → merge children meshes (offset indices)
     ├── 2. Execute CSG operations
     │    • Convert to csgrs Mesh (triangle_mesh_to_csg)
     │    • Boolean via csgrs (union / difference / intersection)
     │    • Convert back (csg_to_triangle_mesh)
     └── Produces: TriangleMesh (ready for renderer or export)
               │
          ┌────┴────────────────────────┐
          │                             │
          ▼                             ▼
   kpe-fabrication               External Renderer / Export
   ├── CutList                   ├── Three.js (Web/WASM)
   ├── Nesting                   ├── STEP (AP242 tessellation)
   └── DXF / SVG                 └── DXF (3DFACE entities)
```

## Materials — World-scale UVs

```
MDF Board (600mm × 2100mm)           MDF Board (300mm × 800mm)
┌─────────────────────────┐          ┌──────────┐
│ ∿∿∿∿∿∿∿∿∿∿∿∿∿∿∿∿∿∿∿∿∿ │          │ ∿∿∿∿∿∿∿ │
│ ∿∿∿∿∿∿∿∿∿∿∿∿∿∿∿∿∿∿∿∿∿ │          │ ∿∿∿∿∿∿∿ │
│ continuous wood grain   │          │ continuous │
│ 1mm texture = 1mm real  │          │ grain    │
└─────────────────────────┘          └──────────┘

With object-relative UVs (INCORRECT):  With world-scale UVs (CORRECT):
Grain scales with the object.         Grain has real-world scale.
A small part has very dense grain.    Same grain density regardless of size.
```
