# ADR-008 — Sketch Nested Profiles, Face Hierarchies and Hole Toggles

**Status:** `proposed`
**Date:** 2026-05-29
**Author:** KPE Engineer

---

## Context

In an advanced CAD Sketch engine, a sketch is rarely a single continuous loop. Users frequently draw shapes inside other shapes (e.g., a square within a square, or multiple concentric circles). In classic CAD paradigms, the innermost loop might be an island, the one outside it a hole, alternating endlessly (Even-Odd rule).

However, in a parametric 3D modeling workflow (like SketchUp or Fusion 360), the user dictates exactly which region forms part of the solid extrusion and which region acts as a void/hole. The system must algorithmically classify outer boundaries and inner boundaries (holes), while exposing an explicit override state to the user so they can enforce a specific topological Face as "Solid" (Drawn/Extrudable) or "Hollow" (Void).

## Options Considered

### Option A — Even-Odd Ray Casting Rule (Implicit Only)
Run a ray-casting intersection test for each discovered Face to count how many parent boundaries enclose it. 
- Odd enclosures = Solid. 
- Even enclosures = Hole.
**Pros:** Fully automatic. No UI needed.
**Cons:** The user has zero explicit control. If they draw a square inside a square and actually want both to be solid (perhaps overlapping), the system forces the inner one to be a hole.

### Option B — Fully Explicit User State Assignment
Treat all discovered faces as solid by default. Require the user to manually click an inner face and toggle "Hollow / Hole" to subtract it.
**Pros:** Maximum user control.
**Cons:** Tedious. 90% of the time, users expect a circle drawn inside a rectangle to act as a extruded hole by default.

### Option C — Hierarchical Face Tree with User Overrides (Hybrid DCEL)
1. **Algorithmic Default:** Using the DCEL Planar Graph (ADR-007), we compute the inclusion hierarchy of all disjoint boundary loops via a point-in-polygon ray-cast from each loop's centroid.
2. **The Topological Tree:** Loops are mounted into a hierarchy: `Root Plane -> Outer Boundary -> Inner Boundary (Hole) -> Island Boundary -> ...`
3. **The Local State Node:** Every discovered Face node in `kpe-schema` contains an explicit `fill_state: Option<SketchFillState>` override property.
4. If `fill_state` is `None`, the system falls back to the Even-Odd level depth. If the user overrides it globally or per-face, it respects their exact will.

**Pros:** 
- Delivers the automated magic of AutoCAD boundary detection.
- Exposes the precise local control of SketchUp planar selection.
- Perfectly predictable for standard 3D Extrusion.

## Decision

**Option C (Hierarchical Face Tree with User Overrides) is chosen.**

This meets the ultra-professional AAA standard. `kpe-geometry` will process the 2D sketch soup into a Doubly Connected Edge List, locate all closed loops, and execute a containment hierarchy algorithm (Parent-Child relationship based on area bounding).

Each localized Face receives an addressable `EntityId`. The user can select the inner square within the UI, triggering an event to alter its `fill_state` flag. When the sweep/extrude command operates, it parses this graph, applying CSG subtractions only for regions evaluated contextually or explicitly as `Hole`.

## Consequences

**Positive:**
- Absolute parametric determinism. If the outer square scales up, the inner square reliably remains a hole unless told otherwise.
- Allows advanced operations: selecting an exact "hole" face to extrude it negatively (CSG difference) against a parent body.

**Negative / Trade-offs:**
- Requires reliable inside-outside topological testing (Point-in-Polygon). Floating-point errors during collinear overlaps (lines exactly on top of other lines) must be resolved with severe algebraic exactness (Epsilon thresholds).

## References
- Requirement: Support a square within a square with explicit Solid/Void toggling.
- Parent Architecture: ADR-007 (Planar Graph Region Extraction).
