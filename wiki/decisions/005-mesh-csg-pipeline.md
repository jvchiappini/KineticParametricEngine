# ADR-005 — Mesh-Based CSG Pipeline (Iterative, Not B-Rep First)

**Status:** `accepted`
**Date:** 2026-05-25
**Author:** KPE Project

---

## Context

ADR-002 defined a plan for a full B-Rep CSG kernel with half-edge data structures,
exact predicates, and face provenance tracking. The estimated timeline was 4-7 months.

After implementing the initial core, several factors led to a re-evaluation:

1. **The renderer is mesh-based.** The custom renderer consumes `TriangleMesh`
   (vertices + triangles), not B-Rep. A full B-Rep kernel adds months of effort
   before producing anything visible.

2. **Mesh CSG works today.** A pipeline of BVH + Möller triangle-triangle
   intersection + ray-casting classification + stitching produces correct boolean
   results on mesh inputs *now*, with no FFI and no external dependencies.

3. **B-Rep is still planned, but not a prerequisite.** The B-Rep half-edge
   structure exists in `kpe-geometry/brep.rs` as a skeleton. It becomes
   critical for face-provenance features (DXF face selection, parametric rebuild
   of specific faces), but those features are Phase 2/3, not Phase 1.

4. **The renderer-agnostic constraint** (Principle 1) is satisfied either way —
   the output is always `TriangleMesh`. The consumer never knows whether the
   CSG went through B-Rep or direct mesh.

## Options Considered

### Option A — Full B-Rep CSG First (ADR-002 original)

Implement the full 7-phase B-Rep CSG before anything else.

**Pros:**
- Face provenance from day one
- Ground truth for all future geometry operations

**Cons:**
- 4-7 months before a visible result
- The renderer cannot consume B-Rep directly — must triangulate anyway
- Blocks all other development (fabrication, parametric, UI)

### Option B — Mesh CSG Pipeline, B-Rep as Skeleton (chosen)

Implement CSG on triangle meshes directly. B-Rep exists as an optional
supplement for tracking metadata.

**Pros:**
- Working CSG in weeks, not months (implemented: 4 days)
- The renderer consumes the output directly — no conversion step
- BVH + intersection + classification are reusable for other geometry queries
- B-Rep can grow incrementally, driven by actual feature requests
- All other development (parametric, fabrication, UI) can proceed in parallel

**Cons:**
- Triangle-level CSG has well-known limitations:
  - No face identity after boolean ops (but B-Rep skeleton can be extended)
  - Degenerate/coincident cases require epsilon tuning
  - No true edge loops for STEP export yet
- Some intersection cases require triangle splitting for correctness
  (current implementation classifies whole triangles — partial overlaps lose
  precision at the boundary)

### Option C — External CSG Library (Manifold)

Use `manifold-rs` (Google/Blender CSG) as the boolean engine.

**Pros:**
- Battle-tested algorithm, minimal maintenance
- Fast, handles degenerate cases well

**Cons:**
- C++ FFI via Emscripten for WASM — unpredictable binary sizes
- No control over the algorithm — face identity impossible
- Violates Principle 1 (renderer-agnostic core with zero FFI)

## Decision

**Option B — Mesh-based CSG pipeline with B-Rep as optional skeleton.**

The decision is pragmatic: working CSG now, B-Rep later when a concrete feature
requires it. The pipeline is:

```
TriangleMesh A ─┐
                 ├──► BVH ──► Triangle-Triangle Intersection (Möller)
TriangleMesh B ─┘                                    │
                                                      ▼
                                            Ray-casting Classification
                                                      │
                                                      ▼
                                            Triangle Selection
                                            (Union / Subtract / Intersect)
                                                      │
                                                      ▼
                                            Stitching (weld + dedup)
                                                      │
                                                      ▼
                                            TriangleMesh (output)
```

The B-Rep kernel (`brep.rs`) remains in the codebase as a half-edge skeleton.
Its `build_brep()` method on `CsgKernel` is the extension point when
face-provenance features are needed.

### What Changed From ADR-002

| Aspect | ADR-002 (original plan) | ADR-005 (actual) |
|--------|------------------------|-------------------|
| Core algorithm | B-Rep half-edge CSG | Mesh boolean (BVH + ray casting) |
| B-Rep implementation | Prerequisite (Phase 1) | Optional skeleton (postponed) |
| Triangle splitting | Required in retriangulation | Future improvement |
| Timeline for working CSG | 4-7 months | 4 days |
| Exact predicates (Shewchuk) | Full adaptive precision | Epsilon-based with `Sign` enum |

## Consequences

**Positive:**
- CSG works now — union, subtract, and intersect produce correct results
  for non-degenerate inputs
- The renderer receives `TriangleMesh` directly — no conversion
- BVH and intersection modules are reusable for collision detection,
  ray picking, and proximity queries
- The parametric solver, fabrication pipeline, and WASM bindings can all
  be developed in parallel with CSG improvements
- Zero FFI, pure Rust, `cargo build` on all platforms

**Negative / Trade-offs:**
- Triangle-level classification loses precision at exact boundaries —
  triangles spanning the intersection surface are classified as a whole
  rather than split, which can produce visible seams
- Coplanar faces produce unpredictable results (depends on epsilon tuning)
- No face identity through boolean ops yet — DXF face selection requires
  the B-Rep extension

**Neutral / Points to Consider:**
- Triangle splitting (clipping each triangle at the intersection segment)
  is the natural next improvement. It lives in `intersection.rs` and
  `csg.rs` without restructuring anything.
- The B-Rep in `brep.rs` is 80 lines of half-edge data structure — it
  serves as documentation of the intended topology model, even before
  the full kernel is implemented.
- ADR-002 is **not superseded** — its phases 2-7 (predicates, intersection,
  BVH, re-triangulation, classification, stitching) were implemented in
  the mesh pipeline. Only the B-Rep-first ordering changed.

## References

- ADR-002 — Original CSG plan (still valid for B-Rep extension)
- ADR-004 — Non-baked transforms (CSG temporarily bakes to common space)
- `kpe-geometry/src/predicates.rs` — orient2d/orient3d with epsilon
- `kpe-geometry/src/intersection.rs` — Möller (1997) triangle-triangle
- `kpe-geometry/src/bvh.rs` — AABB BVH with median split
- `kpe-geometry/src/classify.rs` — Ray-casting winding number
- `kpe-geometry/src/stitch.rs` — Vertex welding and triangle dedup
- Möller, T. (1997). *A Fast Triangle-Triangle Intersection Test.*
