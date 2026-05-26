# ADR-006 — Sketch-Based Parametric Extrusion Pipeline

**Status:** `accepted`
**Date:** 2026-05-26
**Author:** KPE Project

---

## Context

The KPE engine needs to support furniture designs with complex profiles —
chamfered edges, routed grooves, tapered legs, and curved panels — that
cannot be represented by the primitive solids (Box, Cylinder, Sphere)
alone.

Existing approaches:

1. **Manual mesh modelling** — vertex-by-vertex triangle mesh definition.
   Impractical for parametric design; every parameter change requires
   re-modelling.

2. **STEP/DXF import** — loading a pre-authored shape. Works for fixed
   designs but defeats the parametric purpose: dimensions cannot be
   driven by solver rules.

3. **Sketch → Extrude → CSG** — the standard CAD workflow. A 2D sketch
   defines a cross-section, a linear extrusion creates a solid, and CSG
   boolean operations combine or subtract it from other geometry.

Option 3 matches the project's Principle 1 (pure Rust, zero FFI) and
aligns with the existing `GeometryNode` tree design.

## Decision

**Adopt a sketch + extrusion pipeline as the primary profile-authoring
mechanism, with sketches represented declaratively in the recipe JSON.**

The pipeline:

```
SketchDef (primitives in a 2D plane)
        │
        ▼
tessellate_sketch() → Vec<Vec<DVec2>>
  (closed 2D contours)
        │
        │   ExtrudeDef (distance, direction, cap flag)
        ▼
extrude_sketch() → TriangleMesh
  (bottom cap + top cap + side walls)
        │
        ▼
MeshBuilder.build_from_node()
  (merged with scene tree, fed into CSG pipeline)
```

### Schema Types

```rust
pub struct SketchDef {
    pub primitives: Vec<SketchPrimitive>,
    pub plane: SketchPlane,   // XY | XZ | YZ
}

pub enum SketchPrimitive {
    Rectangle { x: f64, y: f64, width: f64, height: f64 },
    Circle    { cx: f64, cy: f64, radius: f64, segments: Option<u32> },
    Polygon   { points: Vec<[f64; 2]> },
    Arc       { cx: f64, cy: f64, radius: f64,
                start_angle: f64, end_angle: f64, segments: Option<u32> },
}

pub struct ExtrudeDef {
    pub sketch_id: String,     // references a Sketch node by id
    pub distance: f64,          // extrusion depth
    pub direction: Option<[f64; 3]>,  // defaults to plane normal
    pub cap: bool,              // close top and bottom
}
```

### Implementation

- `SketchDef` and `ExtrudeDef` live in `kpe-schema/src/geometry.rs`.
- `tessellate_sketch()` in `kpe-geometry/src/sketch.rs` converts primitives
  to `Vec<Vec<DVec2>>` (one closed polyline per contour). Circles and arcs
  are discretised to `segments` vertices (32 and 16 by default).
- `extrude_sketch()` in `kpe-geometry/src/extrude.rs`:
  1. Projects each 2D contour vertex to 3D via `SketchPlane`.
  2. Generates bottom indices (fan from vertex 0, reversed winding).
  3. Generates top indices (fan from vertex 0).
  4. Generates side wall quad strips (2 triangles per quad).
  5. Returns a `TriangleMesh` with no normals (computed on the
     renderer side) and empty UVs.
- `MeshBuilder::build_from_node()` handles `GeometryNodeType::Sketch`
  and `GeometryNodeType::Extrude`. Extrude nodes call `find_sketch()`
  to locate their referenced `SketchDef` in the scene tree by node `id`.

### Limits & Future Work

| Feature | Current | Planned |
|---------|---------|---------|
| Profile types | Rectangle, Circle, Polygon, Arc | Spline, Bezier, Text |
| Sketch plane | XY / XZ / YZ | Arbitrary plane via transform |
| Constraints | None (free-form) | Dimensional constraints (length, angle, coincidence) |
| Revolve | Not implemented | Sketch rotated around an axis |
| Sweep | Not implemented | Sketch swept along a 3D path |
| Loft | Not implemented | Interpolated profile between two sketches |
| Boolean with extrusions | Works via CSG pipeline | Natively merged edge loops |

## Consequences

**Positive:**
- Furniture profiles (rounded corners, tapered legs, routed grooves)
  are now feasible in pure JSON recipes.
- The sketch representation is declarative and serialisable — it can be
  stored as part of `KPERecipe` and version-controlled.
- Extrusion output is a standard `TriangleMesh` that feeds directly into
  the CSG pipeline for subtract/union/intersect.
- No external CAD kernel; zero FFI.

**Negative / Trade-offs:**
- Sketches must be pre-tessellated (discrete vertices). High-precision
  profiles require many segments, increasing memory and triangle count.
- No parametric constraints yet — users must compute sketch coordinates
  in the solver's expression language.
- No filleting or chamfering — those require edge-awareness that the
  current mesh pipeline does not provide.

## References

- ADR-005 — Mesh-based CSG pipeline (extrusion output flows into this)
- ADR-003 — Schema design decisions
- `kpe-schema/src/geometry.rs` — `SketchDef`, `ExtrudeDef`, `SketchPrimitive`
- `kpe-geometry/src/sketch.rs` — `tessellate_sketch()`
- `kpe-geometry/src/extrude.rs` — `extrude_sketch()`
- `kpe-geometry/src/mesh.rs` — `MeshBuilder::build_from_node()`
