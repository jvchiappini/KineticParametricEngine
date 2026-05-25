# KPE Architecture — Diagrams

## Layer Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                          APPLICATIONS                           │
│                                                                 │
│   apps/desktop (Tauri)           apps/web (React + WASM)        │
│   ┌─────────────────┐            ┌─────────────────────┐        │
│   │  UI (React/TS)  │            │   UI (React/TS)     │        │
│   │  Tauri commands │            │   useKPE() hook     │        │
│   └────────┬────────┘            └──────────┬──────────┘        │
│            │ Native FFI                     │ WASM              │
└────────────┼────────────────────────────────┼───────────────────┘
             │                                │
┌────────────┼────────────────────────────────┼───────────────────┐
│            ▼                kpe-wasm        ▼                   │
│   ┌─────────────────────────────────────────────────┐           │
│   │           WASM Bindings (wasm-bindgen)          │           │
│   │  Converts between Rust ↔ JavaScript types       │           │
│   └──────────────────────┬──────────────────────────┘           │
└──────────────────────────┼──────────────────────────────────────┘
                           │
┌──────────────────────────┼──────────────────────────────────────┐
│                          ▼          CORE (Pure Rust)            │
│                                                                 │
│   ┌──────────────────────────────────────────────────────────┐  │
│   │                     kpe-schema                            │  │
│   │   KPERecipe, BlockDefinition, GeometryNode, Joint,       │  │
│   │   Constraint, ProceduralMaterial, CutPiece, ...          │  │
│   │   (Shared types — the contract between layers)           │  │
│   └──────────────────────────────────────────────────────────┘  │
│              ↑               ↑              ↑             ↑      │
│   ┌──────────┴──┐   ┌────────┴───┐   ┌──────┴──────┐  ┌──┴───┐   │
│   │kpe-parametric│   │kpe-geometry│   │kpe-fabricat.│  │kpe-mat.│   │
│   │              │   │            │   │             │  │        │   │
│   │ • solver     │   │ • brep     │   │ • cutlist   │  │ • gen  │   │
│   │ • expression │   │ • csg      │   │ • nesting   │  │ • uv   │   │
│   │ • condition  │   │ • transform│   │ • grain     │  │ • inst │   │
│   │ • rule_engine│   │ • joint    │   │ • dxf/svg   │  │ • text │   │
│   │ • catalog    │   │ • mesh     │   │             │  │        │   │
│   └──────────────┘   └──────┬─────┘   └─────────────┘  └────────┘   │
│                             │                                    │
│              ┌──────────────┴──────────────────┐                 │
│              │  CSG Pipeline (Custom, see      │                 │
│              │  ADR-005)                       │                 │
│              │  ┌──────────┐                   │                 │
│              │  │ BVH SAH │                   │                 │
│              │  └────┬─────┘                   │                 │
│              │       ▼                         │                 │
│              │  ┌──────────────┐               │                 │
│              │  │ Möller TriTri│               │                 │
│              │  └──────┬───────┘               │                 │
│              │         ▼                       │                 │
│              │  ┌──────────────┐               │                 │
│              │  │ Ray-cast     │               │                 │
│              │  │ Classify     │               │                 │
│              │  └──────┬───────┘               │                 │
│              │         ▼                       │                 │
│              │  ┌──────────────┐               │                 │
│              │  │ Stitch + Weld│               │                 │
│              │  └──────────────┘               │                 │
│              └─────────────────────────────────┘                 │
└──────────────────────────────────────────────────────────────────┘
```

## Primary Data Flow

```
User edits parameters in UI
              │
              ▼
        KPERecipe (JSON)
        {
          params: { width: 600, height: 2100 },
          rules: [...],
          geometry: {...},
          joints: {...}
        }
              │
              ▼
    kpe-parametric::Solver
    ├── 1. Evaluate variables:   inner_width = 600 - 2*18 = 564
    ├── 2. Evaluate conditions: width(600) < 800 → no reinforcement
    ├── 3. Apply rules:          has_holes → add subtract operation
    └── Produces: ResolvedRecipe (all concrete values)
              │
              ▼
    kpe-geometry::Builder
    ├── 1. Constructs meshes in local space
    ├── 2. Executes CSG operations (custom pipeline: BVH → Möller → Classify → Stitch)
    ├── 3. Computes world matrices (non-baked)
    ├── 4. Resolves joints and constraints
    └── Produces: GeometryOutput {
                   mesh: TriangleMesh,        → for 3D renderer
                   brep: BRepModel,           → for future operations
                   world_matrices: HashMap,   → for the renderer
                   outline_2d: Sketch2D,      → for 2D renderer / DXF
                 }
              │
         ┌────┴────────────────────────┐
         │                             │
         ▼                             ▼
  kpe-fabrication               External Renderer
  ├── CutList                   (Three.js / wgpu /
  ├── Nesting                    Canvas2D / etc.)
  └── DXF / SVG
```

## Scene Tree — Non-baked

```
Node "cabinet" (compound)
│   world_matrix: Identity
│
├── Node "left_side_panel" (box: 18×2100×600mm)
│   world_matrix: translate(-300, 0, 0)
│   geometry: local_space (no transform applied to vertices)
│
├── Node "right_side_panel" (box: 18×2100×600mm)
│   world_matrix: translate(300, 0, 0)
│
├── Node "door" (box: 596×2096×18mm)
│   world_matrix: translate(0, 0, 318)
│   joint_id: "hinge_top"
│          │
│          └── Joint "hinge_top" (revolute, Y-axis)
│              current_value: 45°   ← changes at runtime
│              limits: { min: 0, max: 110 }
│              "door" world_matrix = translate * rotate(45°, pivot)
│              ↑ recalculated only when current_value changes
│              ↑ NO geometry reconstruction
│
└── CSG Operations on "left_side_panel" (custom pipeline):
    └── subtract: array of ⌀32 holes every 32mm along Y
        BVH → TriTri Intersection → Classify → Stitch
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