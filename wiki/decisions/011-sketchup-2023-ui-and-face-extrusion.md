# ADR-011 — SketchUp 2023+ UI Redesign & Face-Based Extrusion

**Status:** `accepted`
**Date:** 2026-06-04
**Author:** KPE Engineering Team

---

## Context

The original KPE desktop UI (Bevy 0.15 + egui) followed a traditional parametric CAD layout: toolbar at top, scene tree on the left, properties on the right, viewport in the center. While functional, this layout differs dramatically from SketchUp's streamlined interface.

Additionally, the push-pull extrusion system (`kpe-geometry/src/push_pull.rs`) operates on **individual triangles** rather than complete faces, producing results that are non-manifold and unlike SketchUp's clean face extrusion.

The user has requested:

1. **UI/UX identical to SketchUp Pro 2023+**: vertical toolbar, measurements box, minimal panels, full-viewport 3D view
2. **Perfect face extrusion**: extrusion must operate on real faces (boundary loops of coplanar triangles), not individual triangles
3. **.kpe file format preserved**: all models save/load as `.kpe`

## Options Considered

### Option A — Incremental: Keep current structure, fix face extrusion only

Minimal changes: add face detection to kpe-geometry, update extrude_face, keep existing UI layout.

**Pros:**
- Fastest path
- Low risk

**Cons:**
- UI remains dissimilar to SketchUp
- Accumulating technical debt in monolithic files
- Does not satisfy user's requirement for SketchUp-identical UI

### Option B — Full Rewrite: New renderer, new UI framework

Abandon Bevy + egui, use wgpu directly + a custom immediate mode UI.

**Pros:**
- Complete control
- Can match SketchUp pixel-perfectly

**Cons:**
- Massive effort (months)
- Loses Bevy's ECS, asset pipeline, PBR renderer
- No practical benefit for the geometry engine

### Option C — Structured Evolution: New architecture with Bevy (Chosen)

Keep Bevy + egui as the foundation, but:

1. **Restructure `apps/desktop`**: split monolithic `build_tool/mod.rs` into one file per tool with a shared `Tool` trait
2. **Add face detection** to kpe-geometry: group coplanar-connected triangles into `Face` structs with boundary loops
3. **Rewrite push_pull**: extrude along boundary loops, producing clean manifold geometry
4. **SketchUp-style UI**: vertical toolbar overlay, measurements box, minimal panels
5. **Enhanced viewport**: axis-colored inference lines, grid, SketchUp camera behavior

## Decision

**Option C was chosen.** The key design decisions are:

### 1. Face Detection Algorithm

A new module `kpe-geometry/src/face.rs` implements:

- **Edge adjacency**: Build a hash map from `(min_vert, max_vert)` to list of triangle indices
- **Flood-fill face growing**: Start from a seed triangle, add adjacent triangles whose normals are within epsilon (cosine angle > 0.999)
- **Boundary extraction**: Edges that belong to exactly one triangle in the face form the boundary; they are sorted into a closed loop
- **Face struct**: `{ triangle_indices: Vec<usize>, boundary_loop: Vec<(u32, u32)>, normal: DVec3 }`

### 2. Face-Based Extrusion

The existing `extrude_face()` is replaced with:

- Input: `&TriangleMesh`, `&Face`, `distance: f64`
- Output: new `TriangleMesh` with:
  - Original face triangles removed
  - Side faces generated from each boundary edge (two triangles per edge)
  - Top face (original face translated by `normal * distance`)
- Manifold guarantee: every boundary edge produces exactly two triangles forming a quad

### 3. Tool Architecture

Each tool becomes a separate file in `apps/desktop/src/tools/`:

- `mod.rs` — `Tool` trait, `ToolState` resource
- `select.rs` — AABB picking
- `rectangle.rs` — BoxDef creation
- `circle.rs` — CylinderDef creation
- `line.rs` — polyline (future)
- `push_pull.rs` — face detection + extrusion
- `move_tool.rs` — translate nodes
- `eraser.rs` — delete nodes

### 4. UI Layout (SketchUp 2023+)

- **Toolbar**: Vertical strip on the left, icon-only buttons with tooltips (egui `SidePanel::left`)
- **Viewport**: Occupies remaining space, full-bleed
- **Measurements box**: Bottom-right overlay showing dimensions and inference text
- **Default Tray** (optional): `SidePanel::right` with Entity Info, Materials (collapsible)
- **Status bar**: Thin strip at bottom with triangle count and FPS
- **Top menu**: File, Edit, View, Tools, Help (minimal)

### 5. Viewport Enhancements

- **Axis colors**: Red (X), Green (Y), Blue (Z) for inference lines and axis indicators
- **Grid**: Ground plane grid with perspective
- **Camera**: Smoother orbit, zoom-to-extents on key F

## Consequences

**Positive:**
- Face extrusion produces clean, manifold geometry matching SketchUp behavior
- UI becomes immediately recognizable to SketchUp users
- Tool architecture is extensible (future: Follow Me, Offset, Tape Measure)
- Existing crates remain untouched — no regression risk
- .kpe file format unchanged

**Negative / Trade-offs:**
- Face detection adds complexity to the geometry pipeline
- Vertical toolbar requires custom egui rendering for icons
- Some existing UI panels (scene tree, properties) remain visible during transition

**Neutral:**
- The 2D sketch editor persists for complex profiles
- BuildTool keyboard shortcuts remain the same

## References

- ADR-010: SketchUp-Style Direct 3D Manipulation (original BuildTool system)
- `crates/kpe-geometry/src/push_pull.rs` — current triangle-level extrusion
- `apps/desktop/src/build_tool/mod.rs` — current monolithic tool system
- SketchUp Pro 2023 UI layout (industry reference)
