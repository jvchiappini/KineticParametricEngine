# ADR-002 — Custom CSG Implementation over Own B-Rep Kernel

**Status:** `accepted`
**Date:** 2025-05-25
**Author:** KPE project

---

## Context

KPE needs boolean 3D operations (union, subtract, intersect) as a core capability.
Every non-trivial piece produced by the parametric engine goes through at least one
CSG operation — holes, slots, chamfers, compound assemblies.

Two requirements came in together and are inseparable:

1. **Full control over the algorithm** — we need to know exactly which face is which
   after a boolean operation. A black-box CSG library gives us a triangle soup with
   no identity. Our B-Rep model needs to track face provenance through boolean ops
   so that subsequent operations can reference specific faces, edges can be used as
   sketch planes, and cut lists can identify individual surfaces.

2. **No external dependencies for core geometry** — `kpe-geometry` must compile to
   native and WASM with zero C/C++ FFI. Every external C++ CSG library (Manifold,
   CGAL, OpenCASCADE) brings FFI complexity, unpredictable WASM binary sizes, and
   build toolchain requirements we don't control.

These two requirements together rule out every existing library. A black-box CSG
library cannot satisfy (1) even if it satisfies (2). OpenCASCADE satisfies (1) but
catastrophically fails (2). Manifold fails both — it is a mesh-level library with no
face identity, and its WASM story depends on Emscripten.

---

## Options Considered

### Option A — Manifold (Google/Blender)

Mesh-level CSG. Fast, production-proven, used by Blender.
Rust bindings exist (`manifold-rs`).

**Pros:**
- Production-proven (Blender, millions of users)
- Very fast (internal parallelism)
- Rust bindings available

**Cons:**
- Works on triangle soups — no face identity after boolean ops
- Cannot satisfy requirement (1) at all — face provenance is lost
- `manifold-rs` bindings are thin wrappers around C++, FFI required
- WASM compilation goes through Emscripten, not native Rust WASM
- We would need to reimplement everything anyway when we need B-Rep features

### Option B — OpenCASCADE (OCCT)

Real B-Rep kernel. Used by FreeCAD, Salome, dozens of industrial CADs.

**Pros:**
- True B-Rep with complete topology (faces have identity, edges are addressable)
- 30 years of development, extremely reliable
- Native STEP/IGES export
- Satisfies requirement (1) completely

**Cons:**
- Completely fails requirement (2): C++ only, no native Rust bindings, WASM is
  theoretically possible but produces >100MB binaries with known limitations
- `opencascade-rs` bindings are unmaintained and incomplete
- API is extremely verbose — a simple box subtraction is 40+ lines of C++ idioms
- Build toolchain is a significant burden (CMake, C++17, platform-specific flags)

### Option C — Custom CSG over Custom B-Rep (chosen)

Implement CSG operations directly on KPE's own B-Rep representation, in pure Rust.
The algorithm is well-studied — we implement it following the academic literature,
starting with correctness and expanding robustness iteratively.

**Pros:**
- Complete control over face identity and provenance through boolean ops
- Pure Rust — compiles to native and WASM with `cargo build` and `wasm-pack`
- Zero FFI, zero C++ toolchain requirements
- The B-Rep and the CSG are one coherent system, not two separate things glued together
- We understand every line of the geometry code — no black box
- Bugs are our bugs: findable, fixable, testable
- The implementation grows with the project's actual needs, not a superset

**Cons:**
- Significant implementation effort: 4-9 months for a robust implementation
- Floating point robustness is genuinely hard (requires exact arithmetic predicates)
- Edge cases in geometry are numerous and subtle
- We carry the maintenance burden permanently

---

## Decision

**Option C — Custom CSG over own B-Rep kernel, implemented in pure Rust.**

Requirements (1) and (2) are not negotiable and are jointly unsatisfiable by any
existing library. That makes the decision straightforward, even though it is the
hardest path to execute.

The implementation follows established academic algorithms, in this order of phases:

**Phase 1 — B-Rep foundation** (`kpe-geometry/brep.rs`)

A half-edge data structure. Every face, edge, and vertex has a stable `Id` that
survives boolean operations. Face provenance is tracked as metadata.

```
HalfEdge data structure:
  Vertex { id: VertexId, position: Vec3 }
  HalfEdge { id: HalfEdgeId, vertex: VertexId, face: FaceId, twin: HalfEdgeId, next: HalfEdgeId }
  Face { id: FaceId, half_edge: HalfEdgeId, metadata: FaceMetadata }

FaceMetadata {
  source_solid: SolidId,       // which solid this face came from
  source_face: FaceId,         // original face before the boolean op
  operation: Option<CsgOpId>,  // which CSG operation produced this face (if any)
}
```

**Phase 2 — Exact arithmetic predicates** (`kpe-geometry/predicates.rs`)

Robust geometric predicates following Shewchuk (1997): `orient2d`, `orient3d`,
`in_circle`, `in_sphere`. These use adaptive precision arithmetic to guarantee
correct sign results regardless of floating point rounding. This is the foundation
that makes the rest of the algorithm reliable.

**Phase 3 — Triangle-triangle intersection** (`kpe-geometry/intersection.rs`)

Möller (1997) algorithm with exact predicate guards. Handles all 9 intersection
cases between two triangles in 3D. Output is a line segment or a point, with
exact endpoint positions.

**Phase 4 — BVH for spatial acceleration** (`kpe-geometry/bvh.rs`)

An AABB (axis-aligned bounding box) BVH tree using SAH (Surface Area Heuristic)
partitioning. Reduces intersection queries from O(n²) to O(n log n).
Without this, CSG on meshes with >1000 triangles is unusably slow.

**Phase 5 — Face re-triangulation** (`kpe-geometry/retriangulation.rs`)

When two faces intersect, both must be re-triangulated incorporating the intersection
segments. Uses constrained Delaunay triangulation (Shewchuk's Triangle algorithm
adapted for our needs) to produce non-degenerate triangles.

**Phase 6 — Inside/outside classification** (`kpe-geometry/classify.rs`)

For a subtract operation: which faces of mesh A are inside mesh B (to be removed),
and which faces of mesh B are inside mesh A (to be kept as the hole surface)?
Uses winding number classification — more robust than simple ray casting for
non-manifold inputs.

**Phase 7 — Mesh stitching** (`kpe-geometry/stitch.rs`)

Assembles the classified and re-triangulated faces into a closed, manifold output
mesh. Welds vertices within epsilon tolerance. Verifies the output is manifold
(every edge has exactly two adjacent faces).

---

## Implementation Timeline

This is not a sprint — it is a multi-month foundational effort.

| Phase | Estimated effort | Dependency |
|-------|-----------------|------------|
| B-Rep + half-edge | 2-3 weeks | — |
| Exact predicates | 1-2 weeks | — |
| Triangle-triangle intersection | 2-3 weeks | Predicates |
| BVH | 1-2 weeks | — |
| Re-triangulation | 3-5 weeks | Intersection, BVH |
| Classification | 2-3 weeks | BVH, Predicates |
| Stitching | 2-4 weeks | All above |
| **Total (optimistic)** | **~4 months** | |
| **Total (realistic)** | **~7 months** | |

During this period, simple geometry (boxes, cylinders, spheres) and transforms
work without CSG. The parametric solver, the UI, and the fabrication pipeline
can be developed in parallel — they do not depend on CSG being done.

---

## Robustness Strategy

Floating point is the central challenge in computational geometry. Our strategy:

1. **Exact predicates for topology decisions** — any decision that determines
   the *structure* of the output (is this point inside or outside? do these
   segments intersect?) uses exact arithmetic. We never use `a < b` for
   a geometric predicate — we use `orient3d(a, b, c, d).is_positive()`.

2. **Floating point for positions** — actual vertex coordinates use `f64`.
   The error from f64 position computation is bounded and acceptable for
   millimeter-precision CAD. We only need exactness for sign decisions.

3. **Epsilon-based deduplication** — vertices within `1e-6 mm` are considered
   identical during stitching. The epsilon is configurable per `KPERecipe`
   via the `precision` field.

4. **Property-based testing** — every phase has randomized tests that verify
   geometric invariants (output is manifold, face count is correct, volume is
   preserved within tolerance). We use `proptest` for this.

---

## What This Gives Us That No Library Can

Once the CSG is implemented over our B-Rep:

- **Face references survive boolean ops** — a sketch can be drawn on "the top
  face of this box after subtracting that cylinder", and the reference remains
  valid even if the cylinder changes size.

- **Edge loops are first-class** — the boundary of a CSG result is a set of
  addressable half-edges, not just a set of triangles. This is required for
  generating accurate DXF cut lines.

- **Operation history is structural** — we know that face F came from solid A,
  was modified by operation Op3, and its surface normal was flipped during
  the subtract. This powers the parametric rebuild: when a parameter changes,
  we know exactly which faces need to be recomputed.

- **STEP export becomes possible** — STEP requires B-Rep, not meshes. With our
  own B-Rep kernel, STEP export is a serialization problem, not an architectural one.

---

## Consequences

**Positive:**
- Requirements (1) and (2) are both satisfied — face identity and pure Rust
- Complete understanding of every geometry operation in the codebase
- No FFI, no C++ toolchain, `cargo build` is the entire build system
- Foundation for future B-Rep features: fillets, chamfers, shell operations
- STEP/DXF export is architecturally clean

**Negative / trade-offs:**
- Significant upfront investment before CSG is usable
- We carry the maintenance burden for this code permanently
- Robustness bugs in computational geometry are subtle and hard to reproduce
- Community contributions to this module require computational geometry knowledge

**Neutral / watch out for:**
- Simple geometry (no CSG) works from day 1 — don't block other development on this
- The `kpe-geometry/csg.rs` module is the most complex in the codebase by far —
  it must be exceptionally well documented and tested
- Any change to `brep.rs` (the half-edge structure) is a breaking change that
  cascades through the entire geometry pipeline — treat it like a schema change

---

## References

- Shewchuk, J.R. (1997). *Adaptive Precision Floating-Point Arithmetic and Fast Robust Geometric Predicates.* Discrete & Computational Geometry.
- Möller, T. (1997). *A Fast Triangle-Triangle Intersection Test.* Journal of Graphics Tools.
- Laidlaw, D.H., Trumbore, W.B., Hughes, J.F. (1986). *Constructive Solid Geometry for Polyhedral Objects.* SIGGRAPH.
- Zhou, Q., Grinspun, E., Zorin, D., Jacobson, A. (2016). *Mesh Arrangements for Solid Geometry.* ACM Transactions on Graphics. (basis for Manifold's algorithm — useful reference even though we implement our own)
- de Berg, M. et al. *Computational Geometry: Algorithms and Applications.* 3rd ed. Springer. (chapters 6, 11, 13)