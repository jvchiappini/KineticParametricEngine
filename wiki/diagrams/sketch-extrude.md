# Sketch + Extrude Pipeline

## Overview

The sketch + extrude subsystem converts declarative 2D profiles
(`SketchDef`) into 3D triangle meshes (`TriangleMesh`) via linear
extrusion. It is the primary mechanism for authoring furniture
cross-sections — chamfered edges, routed grooves, tapered legs,
curved panels — without resorting to manual vertex data.

## Modules

| Module | Crate | Responsibility |
|--------|-------|----------------|
| `geometry::SketchDef` | `kpe-schema` | Schema type: primitives + plane |
| `geometry::ExtrudeDef` | `kpe-schema` | Schema type: sketch reference + distance |
| `sketch::tessellate_sketch()` | `kpe-geometry` | Primitives → closed 2D polylines |
| `extrude::extrude_sketch()` | `kpe-geometry` | 2D contours → 3D triangle mesh |
| `mesh::MeshBuilder` | `kpe-geometry` | Dispatches Sketch/Extrude nodes in scene tree |

## Data Flow

```
KPERecipe (JSON)
  scene:
    children:
      - id: "profile"
        node_type:
          Sketch:
            primitives:
              - Rectangle { x, y, width, height }
              - Arc { cx, cy, radius, start_angle, end_angle }
            plane: XY
      - id: "part"
        node_type:
          Extrude:
            sketch_id: "profile"
            distance: 50.0
            cap: true
                  │
                  ▼
       MeshBuilder::build_from_node(Extrude)
                  │
                  ├── find_sketch("profile", scene) → SketchDef
                  │
                  ▼
       sketch::tessellate_sketch(SketchDef)
                  │
                  ├── Rectangle → [4 pts]
                  ├── Circle → N pts (default 32)
                  ├── Polygon → M pts
                  └── Arc → K pts (default 16)
                  │
                  ▼
       extrude::extrude_sketch(contours, ExtrudeDef)
                  │
                  ├── Project 2D → 3D via plane normal
                  ├── Bottom cap (fan, reversed winding)
                  ├── Top cap (fan)
                  └── Side walls (quad strips)
                  │
                  ▼
              TriangleMesh
```

## Schema Types

```rust
pub struct SketchDef {
    pub primitives: Vec<SketchPrimitive>,
    pub plane: SketchPlane,
}

pub enum SketchPlane { XY, XZ, YZ }

pub enum SketchPrimitive {
    Rectangle { x: f64, y: f64, width: f64, height: f64 },
    Circle { cx: f64, cy: f64, radius: f64, segments: Option<u32> },
    Polygon { points: Vec<[f64; 2]> },
    Arc { cx: f64, cy: f64, radius: f64,
          start_angle: f64, end_angle: f64, segments: Option<u32> },
}

pub struct ExtrudeDef {
    pub sketch_id: String,
    pub distance: f64,
    pub direction: Option<[f64; 3]>,
    pub cap: bool,
}
```

## Extrusion Algorithm

For each closed 2D contour with N vertices:

```
Bottom cap (cap enabled):
  for i in 1..N-1:
    tri = [0, i+1, i]         // clockwise winding

Top cap (cap enabled):
  for i in 1..N-1:
    tri = [N, N+i, N+i+1]     // counter-clockwise

Side walls:
  for i in 0..N:
    next = (i + 1) % N
    tri0 = [bottom_i, bottom_next, top_next]
    tri1 = [bottom_i, top_next, top_i]
```

### Plane Mapping

| Plane | 2D → 3D | Extrusion direction |
|-------|----------|---------------------|
| XY    | (x, y) → (x, y, 0) | +Z (or custom) |
| XZ    | (x, y) → (x, 0, y) | +Y (or custom) |
| YZ    | (x, y) → (0, y, x) | +X (or custom) |

## Example: Grooved Panel

The following recipe creates a rectangular panel with a routed groove
along its centre — a typical cabinet door profile:

```json
{
  "version": "0.1.0",
  "metadata": { "name": "Routed Panel", "tags": [] },
  "blocks": {},
  "scene": {
    "id": "root",
    "node_type": "Compound",
    "children": [
      {
        "id": "panel_face",
        "node_type": {
          "Sketch": {
            "plane": "XY",
            "primitives": [
              { "Rectangle": { "x": -300, "y": -400, "width": 600, "height": 800 } }
            ]
          }
        }
      },
      {
        "id": "groove_profile",
        "node_type": {
          "Sketch": {
            "plane": "XY",
            "primitives": [
              { "Rectangle": { "x": -3, "y": -5, "width": 6, "height": 5 } }
            ]
          }
        }
      },
      {
        "id": "panel",
        "node_type": {
          "Extrude": {
            "sketch_id": "panel_face",
            "distance": 18,
            "cap": true
          }
        },
        "operations": [
          {
            "op_type": "Subtract",
            "tool_id": "groove_route"
          }
        ]
      },
      {
        "id": "groove_route",
        "node_type": {
          "Extrude": {
            "sketch_id": "groove_profile",
            "distance": 10,
            "cap": true
          }
        },
        "transform": { "translation": [0, 0, 8] }
      }
    ]
  },
  "materials": {},
  "joints": [],
  "constraints": [],
  "precision": null
}
```

## Testing

Tests are in `crates/kpe-geometry/src/sketch.rs` and
`crates/kpe-geometry/src/extrude.rs` (to be added — currently validated
via integration with csg tests). Run:

```bash
cargo test -p kpe-geometry
```

## Future Work

| Feature | Priority | Description |
|---------|----------|-------------|
| Revolve | Medium | Sketch rotated around an axis (lathe) |
| Sweep | Low | Profile along a 3D path (handrail) |
| Loft | Low | Morph between two sketch profiles |
| Constraints | High | Dimensional (length, angle, coincidence) |
| Splines | Medium | Bezier / NURBS in sketch profile |
| Arbitrary plane | Low | Sketch oriented via transform node |

## References

- ADR-006 — Sketch + Extrude decision record
- `kpe-schema/src/geometry.rs` — Schema types
- `kpe-geometry/src/sketch.rs` — Tessellation implementation
- `kpe-geometry/src/extrude.rs` — Extrusion implementation
- `kpe-geometry/src/mesh.rs` — MeshBuilder dispatcher
