# Extrude

Pushes a 2D sketch profile along a linear direction to produce a solid.

## Schema

```rust
pub struct ExtrudeDef {
    pub sketch_id: String,
    pub distance: f64,
    pub direction: Option<[f64; 3]>,
    pub cap: bool,
}
```

| Field       | Type            | Description                                      |
|-------------|-----------------|--------------------------------------------------|
| `sketch_id` | String          | ID of the `SketchDef` node to extrude            |
| `distance`  | f64             | Extrusion depth                                  |
| `direction` | Option\<[f64;3]\> | Extrusion direction (default: sketch plane normal) |
| `cap`       | bool            | Generate top/bottom cap triangles                |

When `direction` is `None` the default direction is the sketch plane normal:
- XY → `+Z`
- XZ → `+Y`
- YZ → `+X`

## Algorithm

1. Tessellate all sketch primitives into closed 2D polylines.
2. For each contour:
   - Project vertices to 3D via the sketch plane.
   - Duplicate at `distance` along the direction.
   - Generate side-wall quad strips between consecutive rings.
   - If `cap == true`, fan-triangulate the bottom and top faces.

## Example Recipe

```json
{
  "id": "root",
  "node_type": "Compound",
  "children": [
    {
      "id": "profile",
      "node_type": { "Sketch": {
        "plane": "XY",
        "primitives": [
          { "Rectangle": { "x": -1, "y": -0.5, "width": 2, "height": 1 } },
          { "Circle": { "cx": 0, "cy": 0, "radius": 0.3 } }
        ]
      }}
    },
    {
      "id": "block",
      "node_type": { "Extrude": {
        "sketch_id": "profile",
        "distance": 4.0,
        "cap": true
      }}
    }
  ]
}
```

Produces a 4-unit-long bar with a rectangular cross-section and a circular
through-hole.
