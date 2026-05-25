# ADR-003 — kpe-schema as a Contract Between Layers

**Status:** `accepted`
**Date:** 2025-05-25
**Author:** KPE Project

---

## Context

KPE consists of multiple layers (`parametric`, `geometry`, `fabrication`, `material`, `wasm`) that need to exchange data. The question is: how can they communicate without becoming tightly coupled?

## Decision

**There is a separate crate, `kpe-schema`, that defines all shared types. No crate imports directly from another to access types—all crates import from `kpe-schema`.**

```
kpe-parametric ──┐
kpe-geometry   ──┤──→ kpe-schema (shared types)
kpe-fabrication──┤
kpe-material   ──┘
```

## Consequences

**Positive:**
- Changing the internal implementation of any crate does not break the others.
- The schema is the single source of truth for type definitions.
- Types are serializable (`serde`) by default → easy to debug and persist.

**Negative:**
- Changing the schema is a major decision that can break all crates.
- Requires careful versioning of the schema as the project matures.

**Points to Consider:**
- Breaking changes to the schema require their own ADR.
- The schema is semantically versioned independently from the rest of the workspace.

---

# ADR-004 — Transforms as Matrices, Not Baked into Vertices

**Status:** `accepted`
**Date:** 2025-05-25
**Author:** KPE Project

---

## Context

In the previous system (KPE v2), transformations (translate, rotate) were "baked" directly into the geometry's vertices before applying CSG. This caused a fundamental issue: whenever a joint rotates, the entire geometry has to be rebuilt from scratch.

## Decision

**Scene tree nodes maintain their geometry in local space. Transformations live as `Matrix4` matrices within the nodes, not within the vertices.**

```
Incorrect (v2):
  vertices = apply_transform(original_vertices, transform)  ← bake
  mesh = CSG(vertices, tool)

Correct (v3):
  mesh_local = build_mesh(node)           ← no transform
  world_matrix = compute_world_matrix(node_path)  ← accumulated by the tree
  renderer.set_matrix(mesh_object, world_matrix)  ← applied by the renderer
```

When a joint value changes, only the `world_matrix` of the affected node and its descendants is updated. The geometry is not rebuilt.

## Consequences

**Positive:**
- Real-time joint movement without rebuilding geometry (60fps possible).
- The renderer can perform correct frustum culling using actual matrices.
- Enables implementation of skeletal-style animation over the tree.

**Negative:**
- CSG operations between nodes in different parts of the tree are more complex (they must be transformed into a common coordinate space before the CSG operation).
- The builder code is more complex than the baking approach.

**Points to Consider:**
- `kpe-geometry/transform.rs` is responsible for all matrix computations.
- CSG **does** temporarily bake transformations to execute boolean operations, but the base geometry of the node remains in local space.
- The renderer receives a `GeometryOutput` containing `world_matrices: HashMap<NodeId, Matrix4>`.