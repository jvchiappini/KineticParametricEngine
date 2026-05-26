# Sweep

Extrudes a 2D sketch profile along a **3D path**, producing a mesh that
follows the curve.

## Schema

```rust
pub struct SweepDef {
    pub sketch_id: String,
    pub path: SweepPath,
    pub segments: Option<u32>,   // default 32
    pub cap: bool,
}
```

| Field       | Type              | Default | Description                             |
|-------------|-------------------|---------|-----------------------------------------|
| `sketch_id` | String            | —       | ID of the `SketchDef` node to sweep     |
| `path`      | SweepPath         | —       | The 3D path definition                  |
| `segments`  | Option\<u32\>     | `32`    | Number of steps along the path          |
| `cap`       | bool              | —       | Generate start/end cap triangles        |

## Path Types

### `Linear`

```rust
Linear { direction: [f64; 3], distance: f64 }
```

A straight path along `direction` (unit vector) for `distance`.  Behaves like
extrude but with an explicit 3D direction; the profile stays parallel to
itself.

| Field       | Type        | Description              |
|-------------|-------------|--------------------------|
| `direction` | `[f64; 3]`  | Path direction vector    |
| `distance`  | f64         | Total path length        |

```json
{
  "path": { "Linear": { "direction": [1, 1, 1], "distance": 5 } }
}
```

### `Arc`

```rust
Arc { radius: f64, angle: f64, axis: [f64; 3] }
```

A circular arc of `radius` sweeping `angle` radians around `axis`.  The
profile is oriented with its local frame following the arc tangent.

| Field    | Type        | Description                      |
|----------|-------------|----------------------------------|
| `radius` | f64         | Arc radius                       |
| `angle`  | f64         | Sweep angle in radians           |
| `axis`   | `[f64; 3]`  | Rotation axis (normalized)       |

```json
{
  "path": { "Arc": { "radius": 3, "angle": 3.14159, "axis": [0, 1, 0] } }
}
```

### `Helix`

```rust
Helix { radius: f64, pitch: f64, turns: f64 }
```

A helical (screw) path.  The profile is oriented with the local frame
following the helix tangent.

| Field    | Type | Description                      |
|----------|------|----------------------------------|
| `radius` | f64  | Helix radius                     |
| `pitch`  | f64  | Vertical distance per full turn  |
| `turns`  | f64  | Number of rotations              |

```json
{
  "path": { "Helix": { "radius": 2, "pitch": 1, "turns": 3 } }
}
```

## Algorithm

1. Tessellate the sketch into closed 2D polylines.
2. Discretise the path into `segments + 1` positions, computing a local
   orthonormal frame (tangent, right, up) at each position.
3. For each path position:
   - Transform the 2D profile vertices to 3D via the sketch plane.
   - Apply the local frame rotation and translation.
4. Generate triangle strips between consecutive frames.
5. If `cap == true`, fan-triangulate the start and end faces.

## Usage Notes

- The profile is always **parallel** to its original sketch plane orientation
  at each path step, adjusted by the local frame rotation.
- For `Linear` paths, the profile remains parallel to itself (constant frame).
- For `Arc` and `Helix` paths, the profile rotates to follow the path tangent.
- High `segments` values produce smoother curves but more triangles.

## Example: Helical Spring

```json
{
  "id": "wire-profile",
  "node_type": { "Sketch": {
    "plane": "YZ",
    "primitives": [
      { "Circle": { "cx": 0, "cy": 0, "radius": 0.15, "segments": 12 } }
    ]
  }},
  "id": "spring",
  "node_type": { "Sweep": {
    "sketch_id": "wire-profile",
    "path": { "Helix": { "radius": 1.5, "pitch": 0.8, "turns": 5 } },
    "segments": 120,
    "cap": false
  }}
}
```

## Example: Curved Handrail

```json
{
  "id": "rail-profile",
  "node_type": { "Sketch": {
    "plane": "YZ",
    "primitives": [
      { "Rectangle": { "x": -0.03, "y": -0.04, "width": 0.06, "height": 0.08 } }
    ]
  }},
  "id": "handrail",
  "node_type": { "Sweep": {
    "sketch_id": "rail-profile",
    "path": { "Arc": { "radius": 2, "angle": 1.5708, "axis": [0, 0, 1] } },
    "segments": 24,
    "cap": true
  }}
}
```
