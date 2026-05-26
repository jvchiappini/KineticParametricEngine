# Sketch System

The KPE sketch system provides a declarative way to define 2D profiles and
turn them into 3D geometry via **Extrude**, **Revolve**, and **Sweep**
operations.

## Architecture

```
SketchPrimitive ──tessellate──▶ 2D Contours ──extrude──▶ TriangleMesh
                                        ├──revolve──▶ TriangleMesh
                                        └──sweep ───▶ TriangleMesh
```

A **sketch** lives on a **sketch plane** (XY, XZ, YZ) and consists of one or
more **primitives** (Rectangles, Circles, Polygons, Arcs).  Each primitive is
tessellated into a closed polyline; the sketch nodes are then passed to an
operation node (Extrude, Revolve, Sweep) that produces a `TriangleMesh`.

## Quick Start

```json
{
  "id": "my-sketch",
  "node_type": {
    "Sketch": {
      "plane": "XY",
      "primitives": [
        { "Rectangle": { "x": -1, "y": -0.5, "width": 2, "height": 1 } }
      ]
    }
  }
}
```

```json
{
  "id": "my-extrude",
  "node_type": {
    "Extrude": {
      "sketch_id": "my-sketch",
      "distance": 3.0,
      "cap": true
    }
  }
}
```

## Reference

| Document | Description |
|----------|-------------|
| [Primitives](primitives.md) | All sketch primitives and their parameters |
| [Extrude](extrude.md) | Linear extrusion |
| [Revolve](revolve.md) | Rotational sweep (lathe) |
| [Sweep](sweep.md) | Sweep along a path |
| [Examples](examples/) | Full recipe examples |
