# Revolve (Lathe)

Rotates a 2D sketch profile around an axis to produce a solid — the
programmatic equivalent of a lathe (torno).

## Schema

```rust
pub struct RevolveDef {
    pub sketch_id: String,
    pub angle: f64,              // radians, full circle = 2π
    pub segments: Option<u32>,   // default 32
    pub axis: RevolveAxis,       // X | Y | Z
    pub cap: bool,
}
```

| Field       | Type              | Default | Description                                |
|-------------|-------------------|---------|--------------------------------------------|
| `sketch_id` | String            | —       | ID of the `SketchDef` node to revolve      |
| `angle`     | f64               | —       | Total rotation angle in radians            |
| `segments`  | Option\<u32\>     | `32`    | Number of angular subdivisions             |
| `axis`      | RevolveAxis       | —       | Axis of rotation (X, Y, or Z)              |
| `cap`       | bool              | —       | Generate end-cap triangles at start/end    |

### `RevolveAxis`

```rust
pub enum RevolveAxis { X, Y, Z }
```

## Algorithm

1. Tessellate the sketch into closed 2D polylines.
2. Project each contour point to 3D via the sketch plane.
3. For each of `segments + 1` angular steps:
   - Rotate the projected contour around the chosen axis by `θ = step × angle`.
4. Generate triangle strips between consecutive rings.
5. If `cap == true` and angle < 2π, fan-triangulate the start and end faces.

## Usage Notes

- A **partial revolve** (angle < 2π) with `cap: true` produces a pie-slice
  solid with flat start/end faces.
- A **full revolve** (angle = 2π = 6.283185) produces a rotationally
  symmetric solid; caps are suppressed automatically.
- Most lathe-like shapes (vases, table legs, wine glasses, bowls) use
  `axis: Y` with the sketch on the XY or XZ plane.

## Example: Turned Table Leg

```json
{
  "id": "leg-profile",
  "node_type": { "Sketch": {
    "plane": "XY",
    "primitives": [
      { "Polygon": { "points": [
        [0.3, 0], [0.3, 0.5], [0.5, 0.7], [0.5, 1.5],
        [0.3, 1.7], [0.3, 2.5], [0.4, 2.7], [0.4, 3.5],
        [0.3, 3.7], [0.3, 4.5], [0.5, 4.7], [0.5, 5.0],
        [0.3, 5.0], [0.3, 5.5], [0, 6.0], [-0.3, 5.5],
        [-0.3, 5.0], [-0.5, 4.7], [-0.5, 4.5], [-0.3, 3.7],
        [-0.3, 3.5], [-0.4, 2.7], [-0.4, 2.5], [-0.3, 1.7],
        [-0.3, 1.5], [-0.5, 0.7], [-0.5, 0.5], [-0.3, 0]
      ]}}
    ]
  }},
  "id": "leg",
  "node_type": { "Revolve": {
    "sketch_id": "leg-profile",
    "angle": 6.283185307179586,
    "segments": 48,
    "axis": "Y",
    "cap": false
  }}
}
```

## Example: Dome (partial revolve)

```json
{
  "id": "arch-profile",
  "node_type": { "Sketch": {
    "plane": "XY",
    "primitives": [
      { "Arc": { "cx": 0, "cy": 0, "radius": 2, "start_angle": 0, "end_angle": 3.1416, "segments": 32 } }
    ]
  }},
  "id": "dome",
  "node_type": { "Revolve": {
    "sketch_id": "arch-profile",
    "angle": 3.141592653589793,
    "segments": 32,
    "axis": "Y",
    "cap": true
  }}
}
```
