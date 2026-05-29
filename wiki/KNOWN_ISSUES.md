# Known Issues and Limitations

## No Feature Tree / Construction History

KPE has no separate feature tree or construction history model. The scene tree *is* the model representation. Operations like fillet and chamfer are represented as wrapper nodes rather than parametric features. This means there is no timeline, no rollback to a specific construction step, and no reorderable feature list.

## CSG Kernel Quality

Boolean operations (Union, Subtract, Intersect) use the `csgrs` crate, which is a BSP-based CSG library. Known issues:
- Z-fighting / coplanar face artifacts when two faces are nearly coincident but not exactly aligned
- Numerical instability with small or thin features
- No support for non-manifold geometry
- Performance degrades with high triangle counts

## Sketch Solver Divergence

The gradient-descent constraint solver can diverge when cyclic constraints create an over-constrained or contradictory system. For example, a triangle with all three side lengths fixed and a perpendicular constraint applied to two edges may cause the solver to oscillate or produce nonsense results. There is no constraint cycle detection.

## Missing Export Formats

| Format | Status |
|---|---|
| STL Binary | Implemented (`io.rs:54`) |
| OBJ Wavefront | Implemented (`io.rs:108`) |
| STEP (ISO 10303) | Not implemented |
| DXF (AutoCAD) | Not implemented |
| SVG (2D vector) | Not implemented |
| 3MF (3D Manufacturing) | Not implemented |
| PDF (2D projection) | Not implemented |

## Sketch Edits Not Undoable

While the sketch editor has its own snapshot-based undo stack, exiting the sketch editor and writing the result back to the scene tree is not wrapped in a `Command`. There is no way to undo a sketch edit from the main document's undo history.

## No Grid Snap

The sketch grid is purely visual. There is no snap-to-grid or snap-to-angle behavior when drawing lines, circles, or arcs. Point placement is continuous.

## Missing Sketch Tools

The sketch editor lacks several common tools:
- Spline / Bezier curve tool
- Trim (cut at intersection)
- Extend (lengthen to next entity)
- Offset (parallel curves)
- Mirror in sketch
- Pattern in sketch (array)

## No Drag-and-Drop in Scene Tree

The scene tree is static — nodes cannot be reordered by dragging. The only way to change parent/child relationships is through cut/paste operations.

## All Evaluation Blocks the Frame

`evaluate_all()` is synchronous and runs on the main thread. For complex scenes with many CSG operations, this can cause frame drops. There is no multi-threaded evaluation, incremental evaluation, or background compute.

## No LOD or Frustum Culling

All meshes are rendered at full detail regardless of distance from the camera. Bevy's built-in frustum culling is active (via `Visibility` components), but there is no level-of-detail switching or mesh simplification.

## File Size: sketch_editor.rs

`apps/desktop/src/sketch_editor.rs` is **735 lines** — exceeding the convention of a 700-line per-file limit. It combines sketch input logic, UI rendering, workflow orchestration, and helper functions in a single file. Refactoring into smaller modules (e.g. separating input, UI, and state management) would improve maintainability.
