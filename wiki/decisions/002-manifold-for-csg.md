# ADR-002 — CSG Engine: csgrs (puro Rust, BSP)

**Status:** `implemented — migrated from manifold-csg to csgrs`
**Date:** 2025-05-25 (last updated: 2026-05-25)
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

---

## Options Considered

### Option A — Manifold (Google/Blender) via `manifold-csg`

Mesh-level CSG backed by a C++ kernel.

**Pros:**
- Production-proven (Blender, millions of users)
- Very fast (internal parallelism)
- Rust bindings available

**Cons:**
- Works on triangle soups — no face identity after boolean ops
- Cannot satisfy requirement (1) at all — face provenance is lost
- `manifold-csg` bindings require C++/CMake toolchain
- WASM compilation goes through Emscripten, not native Rust WASM
- The `manifold-csg` crate (`v0.2.0`) failed to compile on our CI/dev  
  environment due to missing or incompatible C++ toolchain components

### Option B — OpenCASCADE (OCCT)

Real B-Rep kernel. Used by FreeCAD, Salome, dozens of industrial CADs.

**Cons:**
- Completely fails requirement (2): C++ only, no native Rust bindings
- `opencascade-rs` bindings are unmaintained and incomplete
- Build toolchain is a significant burden (CMake, C++17, platform-specific flags)

### Option C — Custom CSG over Custom B-Rep

Implement CSG operations directly on KPE's own B-Rep representation, in pure Rust.

**Pros:**
- Complete control over face identity and provenance
- Pure Rust — zero FFI

**Cons:**
- Significant implementation effort: 4–9 months for a robust implementation
- Floating point robustness is genuinely hard

### Option D — `csgrs` (chosen)

Pure-Rust CSG library using BSP trees. `Polygon/Vertex/Mesh` data model.
Boolean operations: `union`, `difference`, `intersection` via the `CSG` trait.

**Pros:**
- 100% pure Rust — `cargo build` is the entire build system
- No C++ toolchain, no CMake, no Emscripten
- Compiles cleanly to native and WASM targets
- Simple, composable API: `Mesh<S>::union(&other)`, `difference`, `intersection`
- Actively maintained on crates.io (`v0.20.1`)
- BSP algorithm is well-understood and deterministic

**Cons:**
- Mesh-level library — no face provenance (same limitation as Manifold)
- BSP-based CSG can be slower than Manifold on very dense meshes
- Robustness on degenerate/co-planar triangles depends on the BSP implementation
- Requires patching `core2` in the workspace manifest (see below)

---

## Decision (Updated 2026-05-25)

**Replace `manifold-csg` (C++ FFI, failed to compile) with `csgrs` (pure Rust, BSP).**

### Reasoning

1. **Buildability is non-negotiable.** `manifold-csg v0.2.0` could not be compiled
   due to toolchain requirements (CMake / MSVC). The project was blocked.
2. **`csgrs` satisfies requirement (2) completely.** It is pure Rust, compiles with
   `cargo build`, and requires no external toolchain.
3. **Trade-off on requirement (1) is unchanged.** Both `manifold-csg` and `csgrs`
   are mesh-level libraries — neither provides face provenance. This is an accepted
   limitation for the current phase. Full face-identity B-Rep (Option C) remains a
   long-term goal.

### Known Limitation: `core2` patch

`csgrs v0.20.1` transitively depends on `core2 = "^0.4"`, which was yanked from
crates.io. The workspace `Cargo.toml` includes a `[patch.crates-io]` entry that
sources `core2` from the upstream git repository (same revision that `csgrs` used
when declaring the dependency):

```toml
[patch.crates-io]
core2 = { git = "https://github.com/bbqsrc/core2", rev = "545e84bcb0f235b12e21351e0c69767958efe2a7" }
```

This patch can be removed once `csgrs` upgrades its own `core2` dependency or the
package is republished to crates.io.

---

## Implementation

### Architecture (`crates/kpe-geometry/src/csg.rs`)

```
TriangleMesh  →  [triangle_mesh_to_csg]  →  Mesh<()>
                                               │
                                          CSG trait
                                         .union / .difference / .intersection
                                               │
Mesh<()>  →  [csg_to_triangle_mesh]  →  TriangleMesh
```

**Conversion — KPE → csgrs:**
Each triangle in `TriangleMesh` becomes one `Polygon<()>` with three `Vertex`
entries. Vertex normals are computed as the face normal (flat shading). Degenerate
triangles (zero-area) are silently dropped before entering the BSP kernel.

**Conversion — csgrs → KPE:**
Each `Polygon` in the result is triangulated via `Polygon::triangulate()` which
returns `Vec<[Vertex; 3]>`. Vertices are emitted in a flat (non-indexed) layout —
each triangle owns its three vertices independently.

### Dependencies added

| Crate       | Version  | Purpose                        |
|-------------|----------|--------------------------------|
| `csgrs`     | 0.20.1   | CSG boolean operations (BSP)   |
| `nalgebra`  | 0.33     | `Point3` / `Vector3` types     |

`manifold-csg` has been removed entirely.

---

## Implementation Status (as of 2026-05-25)

| Phase | File | Status | Notes |
|-------|------|--------|-------|
| B-Rep foundation | `brep.rs` | ✅ Scaffolded | Basic half-edge structure exists |
| CSG Integration | `csg.rs` | ✅ Done | Replaced manifold-csg with csgrs |
| Tri-Tri Intersection | — | ⏭️ Delegated | Handled by csgrs BSP kernel |
| BVH acceleration | — | ⏭️ Delegated | Handled by csgrs BSP kernel |
| Triangle splitting | — | ⏭️ Delegated | Handled by csgrs BSP kernel |
| Classification | — | ⏭️ Delegated | Handled by csgrs BSP kernel |
| Mesh stitching | — | ⏭️ Delegated | Handled by csgrs BSP kernel |
| WASM Support | — | ✅ Unblocked | Pure Rust, no Emscripten needed |

---

## Consequences

**Positive:**
- Build is unblocked on all platforms (`cargo build` is sufficient)
- WASM compilation is now straightforward (`wasm-pack build`)
- Zero external toolchain requirements (no CMake, no MSVC CRT issues)
- API is clean and idiomatic Rust

**Negative / Trade-offs:**
- No face provenance — same limitation as Manifold. Full B-Rep identity
  requires implementing Option C in the future.
- A `[patch.crates-io]` entry is needed in the workspace for `core2` until
  `csgrs` updates its dependency tree.

**Neutral / watch out for:**
- BSP-based CSG can produce T-junctions on near-coplanar faces. Test with
  intentionally degenerate geometry.
- The flat (non-deduplicated) vertex layout from `csg_to_triangle_mesh` means
  vertex count grows with triangle count. Deduplicate if a downstream consumer
  requires indexed geometry.

---

## References

- csgrs crate: <https://docs.rs/csgrs/latest/csgrs/>
- core2 patch source: <https://github.com/bbqsrc/core2>
- Shewchuk, J.R. (1997). *Adaptive Precision Floating-Point Arithmetic and Fast Robust Geometric Predicates.*
- Laidlaw, D.H., Trumbore, W.B., Hughes, J.F. (1986). *Constructive Solid Geometry for Polyhedral Objects.* SIGGRAPH.
- de Berg, M. et al. *Computational Geometry: Algorithms and Applications.* 3rd ed. Springer.