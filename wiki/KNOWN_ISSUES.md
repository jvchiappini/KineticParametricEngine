# Known Issues and Limitations

## Line/Move/Eraser Build Tools Not Implemented

The `BuildTool` enum has `Line`, `Move`, and `Eraser` variants with keyboard shortcuts (L, M, E) and UI slots in the tool palette, but their behavior is not implemented. `ToolPhase::Polylining` and `ToolPhase::Dragging` are scaffolded but never constructed. This means the SketchUp workflow is incomplete — only Select, Rectangle, Circle, and PushPull work.

## PushPull Operates on Single Triangles

`extrude_face()` in kpe-geometry extrudes a single triangle. There is no coplanar face grouping — if a face consists of multiple triangles, each must be extruded individually. A face grouping algorithm (adjacent coplanar triangle clustering) is needed.

## Parametric Push-Pull Not Implemented

When the PushPull tool acts on a face of a Box or Cylinder, it currently extrudes the mesh. The ideal behavior would be to detect the primitive type and adjust `BoxDef.height` or `CylinderDef.height` directly, preserving parametric editability.

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
| STL Binary | Implemented |
| OBJ Wavefront | Implemented |
| DXF (AutoCAD R12) | Implemented (3DFACE entities) |
| SVG (2D vector) | Implemented (triangle edges) |
| STEP (ISO 10303) | Not implemented |
| 3MF (3D Manufacturing) | Not implemented |
| PDF (2D projection) | Not implemented |

## Sketch Edits Not Undoable

While the sketch editor has its own snapshot-based undo stack, exiting the sketch editor and writing the result back to the scene tree is not wrapped in a `Command`. There is no way to undo a sketch edit from the main document's undo history.

## No Grid Snap in 2D Sketch

The sketch grid has a basic snap-to-grid (toggleable via checkbox, default 0.5 spacing), but there is no snap-to-angle or snap-to-increment mode available in the BuildTool 3D system. The 2D sketch editor has grid snap but it's basic.

## Missing Sketch Tools

The sketch editor lacks several common tools:
- Spline / Bezier curve tool
- Trim (cut at intersection)
- Extend (lengthen to next entity)
- Offset (parallel curves)
- Mirror in sketch
- Pattern in sketch (array)

## No Drag-and-Drop in Scene Tree

Drag-and-drop was implemented in Session 5 (`MoveNodeCommand`) but may have edge cases with nested container reordering. The scene tree supports dragging nodes via `drag_started()`/`drag_stopped()` with visual drop indicators (line above/below, rectangle for AsChild). Cycle prevention via `is_descendant_of()` check.

## All Evaluation Blocks the Frame

`evaluate_all()` is synchronous and runs on the main thread. For complex scenes with many CSG operations, this can cause frame drops. There is no multi-threaded evaluation, incremental evaluation, or background compute.

## No LOD or Frustum Culling

All meshes are rendered at full detail regardless of distance from the camera. Bevy's built-in frustum culling is active (via `Visibility` components), but there is no level-of-detail switching or mesh simplification.

## File Sizes (Near Limit)

The following files are close to or exceed the 700-line convention:

| File | Lines | Notes |
|------|-------|-------|
| `kpe-geometry/src/sketch/solver.rs` | 758 | Exceeds limit; needs splitting (solver core vs constraint types) |
| `kpe-parametric/src/commands/mod.rs` | 586 | Near limit |
| `apps/desktop/src/ui/properties.rs` | 563 | Near limit |
| `apps/desktop/src/ui/scene_tree.rs` | 578 | Near limit |
| `apps/desktop/src/ui/properties.rs` | 563 | Near limit |
| `apps/desktop/src/build_tool/mod.rs` | 486 | Combine with Line/Move/Eraser tools will exceed; plan split |
| `apps/desktop/src/push_pull_system.rs` | 379 | OK, below limit |
| `kpe-geometry/src/evaluator.rs` | 569 | Near limit |

The sketch editor was refactored from a single file into `sketch_editor/` module with separate state/input/ui files (all < 510 lines each).
