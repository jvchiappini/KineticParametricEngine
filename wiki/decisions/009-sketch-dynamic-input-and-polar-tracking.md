# ADR-009 — Sketch Dynamic Input, Polar Tracking, and Explicit Dimension Entries

**Status:** `proposed`
**Date:** 2026-05-29
**Author:** KPE Engineer

---

## Context

A fundamental requirement of AAA CAD sketching engines (like AutoCAD) is that users rarely draw geometry relying solely on visual mouse placements. Accurate industrial drafting relies heavily on explicit numeric dimensioning *while* actively drawing. 

Presently, drawing a line with an exact length (e.g., 50.0mm) precisely at a 45° angle involves awkwardly snapping to a grid or attempting to post-constrain randomly drawn lines. To make the sketch experience truly perfect for engineering, the engine must support explicit override dimensions entered dynamically via the keyboard, combined with intelligent polar alignment tracking.

## Options Considered

### Option A — Post-drawing Parametric Constraints Only
Force users to draw sloppy/approximate geometry and then require them to manually apply `Constraint::Distance` and `Constraint::Angle` post-facto.
**Pros:** Easy to implement. Leverages the existing constraint solver.
**Cons:** Frustrating, tedious UX. Highly detrimental for quick drafting tasks. Not acceptable for industry-standard applications.

### Option B — Dynamic Input HUD & Constrained Polar Inference
Implement an active overlay (HUD) connected directly to the `InferenceEngine`. As the user moves the mouse, the engine checks for polar alignments (ortho tracking every 90° or 45°) and snaps the geometric intent to that line constraint. While drawing, directly typing numeric keys captures distance/angle overrides and injects them instantly into the placement calculation.
**Pros:**
- Ultra-precise, fast drafting workflows.
- Eliminates the need to apply dozens of constraints manually.
- The lines inherently snap to their designated constraints as they are created.
**Cons:**
- High complexity within the UI interaction event loop.
- The `input.rs` and `ui.rs` systems must handle interrupted event states without disrupting the main application inputs.

## Decision

**Option B is chosen to align with AAA architectural demands.**

Drawing entities is no longer a naive click-and-drag. The `InferenceEngine` will compute angular snap offsets dynamically. The user interface overlay must intercept number keys to bypass spatial mouse coordinates. 

When creating a line:
1. `Start point` is clicked.
2. `End point` trails the cursor.
3. If the cursor angle naturally falls within an `epsilon` offset of an architectural threshold (0°, 45°, 90°, etc.), it triggers the **Polar Snapping** override.
4. If the user hits a numeric key (e.g., "120") and presses "Enter" or "Tab", the `End point` distance from the `Start point` is mathematically locked, respecting the active Polar Angle (if snapped) or current free angle.

## Consequences

**Positive:**
- Flawless mathematical vector tracing in absolute and relative coordinates.
- Greatly increases modeling speed for machined parts relying strictly on blueprint dimensions.
- Unifies implicit constraints and drawing creation under one transactional roof.

**Negative / Trade-offs:**
- Requires intercepting raw keyboard events over the standard GUI interaction mapping to spawn temporary entry widgets seamlessly.

## References
- Requirement: Perfect sketch drafting (angles and distances).
- Architecture parent: Sketch Engine Roadmaps.
