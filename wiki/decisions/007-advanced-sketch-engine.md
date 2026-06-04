# ADR-007 — Advanced Sketch Engine with Planar Graph Region Extraction

**Status:** `proposed`
**Date:** 2026-05-29
**Author:** KPE Engineer

---

## Context

The current Sketch system operates as a naive collection of 2D entities. This restricts usability compared to industry-standard CAD systems like AutoCAD (drafting) and SketchUp (direct region manipulation). The primary issues are:
1. **Lack of topological intelligence:** Lines that cross each other do not naturally form sub-regions.
2. **Monolithic extrusions:** Extrusions are applied to the entire sketch profile sequentially, prohibiting selective "push/pull" mechanics on specific faces.
3. **Rigid drafting experience:** True parametric drafting requires real-time inference (Object Snaps) for intersections, midpoints, tangent points, and apparent intersections.

To achieve an AAA-grade user experience, the parametric engine must treat 2D sketches not merely as rendering vectors, but as dynamic topological planar graphs. 

## Options Considered

### Option A — Naive Vector Rasterization (SDF / Clipper)
Use standard 2D boolean clipping libraries (e.g., Clipper2) to union or intersect paths before extrusion.
**Pros:** Easy to implement, works well with standard SVG/DXF pipelines.
**Cons:** Destroys parametric relationships. Cannot effectively identify inner arbitrary intersecting sub-regions dynamically for a SketchUp-like "Push/Pull" UX. Does not preserve exact analytic curves (Arcs).

### Option B — Doubly-Connected Edge List (DCEL) / Planar Graph Topology
Treat the sketch as an analytical 2D plane. Compute all intersection matrices between every curve, split segments at intersections, and walk the graph algorithmically to discover all minimal closed loops (Faces). 
**Pros:**
- Discovers independent regions perfectly, exactly like SketchUp.
- Maintains exact analytic geometry (Arcs remain Arcs, not just line segments).
- Enables user-selectable faces for push/pull operations without explicit profile definition.
**Cons:**
- The algorithmic complexity of dynamic planar graph subdivision robustly handling collinear and tangency edge cases is extremely high.
- Requires a strict and advanced mathematical solver core.

## Decision

**Option B (DCEL / Planar Graph Topology) is chosen.**

To reach industrial AAA quality, we must implement our own planar graph topology engine within `kpe-geometry`. The UX flow dictates that users draw unconstrained intersecting lines (like AutoCAD), and the system automatically delineates hovering regions (like SketchUp). This requires continuous computational geometry evaluations that only a topological graph can provide reliably.

### System Architecture Pipeline
1. **The Inference Engine (OSnap):** Operates on the UI loop. Uses BVH (Bounding Volume Hierarchies) to quickly query distances from the cursor to analytical entities (endpoints, curve centers, parametric midpoints, computed intersections).
2. **The Topological Resolver:** Operates inside `kpe-geometry`. When a sketch is evaluated, all contained curves (Lines, Arcs) are subjected to algebraic intersections. Curves are split into "Half-Edges" at these intersection nodes.
3. **Face Discovery (The Graph Walker):** Traverses the Half-Edges using the "left-most turn" heuristic to identify unique, minimal bounded regions (Faces) and unbounded outer regions (Holes). 
4. **Push/Pull Interactor:** The selected sub-face is structurally isolated as a `GeometryNode` and converted into a standard 3D Extrude operation.

## Consequences

**Positive:**
- Seamless, organic drawing experience.
- The user is freed from manually trimming vectors to create closed profiles. They simply draw, and the engine detects the regions automatically.
- Snapping becomes mathematically precise rather than visually approximated.

**Negative / Trade-offs:**
- Requires significant upfront engineering for intersection algorithms (Line-Line, Line-Arc, Arc-Arc) to handle floating-point precision issues cleanly.
- High computational overhead for high-entity-count sketches unless continuously optimized via Spatial Hash Grids or BVH.

## References
- Requirement: Evolve the sketch system into an AutoCAD drafting + SketchUp push/pull hybrid.
