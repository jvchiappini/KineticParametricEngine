# Sketch Primitives

All primitives are defined inside a `SketchDef.primitives` array and are
tessellated into **closed 2D contours** on the sketch plane.

## `Rectangle`

```rust
Rectangle { x: f64, y: f64, width: f64, height: f64 }
```

An axis-aligned rectangle with its lower-left corner at `(x, y)`.

| Field    | Type  | Description            |
|----------|-------|------------------------|
| `x`      | f64   | Left edge X position   |
| `y`      | f64   | Bottom edge Y position |
| `width`  | f64   | Width (X extent)       |
| `height` | f64   | Height (Y extent)      |

**Tessellation:** 4 vertices, closed quad.

```json
{ "Rectangle": { "x": 0, "y": 0, "width": 2, "height": 1 } }
```

---

## `Circle`

```rust
Circle { cx: f64, cy: f64, radius: f64, segments: Option<u32> }
```

A circle centered at `(cx, cy)`.

| Field      | Type       | Default | Description              |
|------------|------------|---------|--------------------------|
| `cx`       | f64        | —       | Center X                 |
| `cy`       | f64        | —       | Center Y                 |
| `radius`   | f64        | —       | Radius                   |
| `segments` | Option\<u32\> | `64` | Number of edge segments  |

```json
{ "Circle": { "cx": 0, "cy": 0, "radius": 1.5, "segments": 48 } }
```

---

## `Polygon`

```rust
Polygon { points: Vec<[f64; 2]> }
```

An arbitrary closed polygon defined by a list of `(x, y)` vertices.  The
polygon is **automatically closed** (the last vertex connects back to the
first).

| Field    | Type            | Description                 |
|----------|-----------------|-----------------------------|
| `points` | Vec\<[f64; 2]\> | Ordered list of 2D vertices |

```json
{
  "Polygon": {
    "points": [[0,0], [2,0], [2,1], [1,2], [0,1]]
  }
}
```

---

## `Arc`

```rust
Arc { cx: f64, cy: f64, radius: f64, start_angle: f64, end_angle: f64, segments: Option<u32> }
```

A circular arc centered at `(cx, cy)`.  The arc sweeps counter-clockwise from
`start_angle` to `end_angle` (in radians).

| Field         | Type       | Default | Description               |
|---------------|------------|---------|---------------------------|
| `cx`          | f64        | —       | Center X                  |
| `cy`          | f64        | —       | Center Y                  |
| `radius`      | f64        | —       | Radius                    |
| `start_angle` | f64        | —       | Start angle in radians    |
| `end_angle`   | f64        | —       | End angle in radians      |
| `segments`    | Option\<u32\> | `32` | Number of edge segments   |

**Note:** An arc is **not** closed by itself — it produces an open polyline.
To create a closed shape from an arc, combine it with other primitives in the
same `SketchDef` (e.g., chord lines from a polygon) or use a rectangle/polygon
to cap the ends.

```json
{ "Arc": { "cx": 0, "cy": 0, "radius": 2, "start_angle": 0, "end_angle": 3.1416 } }
```

---

## Sketch Plane

```rust
enum SketchPlane { XY, XZ, YZ }
```

Defines which 3D plane the 2D sketch coordinates are projected onto.

| Plane | Mapping              |
|-------|----------------------|
| `XY`  | `(x, y) → (x, y, 0)` |
| `XZ`  | `(x, y) → (x, 0, y)` |
| `YZ`  | `(x, y) → (0, y, x)` |

## Geometry Node Type

```rust
GeometryNodeType::Sketch(SketchDef)
```

When built directly (without Extrude/Revolve/Sweep), a Sketch node returns a
flat `TriangleMesh` with the contour verts projected onto `z=0` and zero
triangles — useful for 2D outlines or previews.
