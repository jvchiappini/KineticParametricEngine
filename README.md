# KPE — Kinetic Parametric Engine

> A general-purpose, open-source parametric CAD designed for extreme ease of use.
> Millimeter-precise geometry, dynamic blocks with conditional logic, and ready for any renderer.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://rustup.rs)
[![WASM](https://img.shields.io/badge/Target-WASM%20%2B%20Native-green.svg)]()
[![CI](https://img.shields.io/github/actions/workflow/status/jvchiappini/KineticParametricEngine/ci.yml?branch=main)]()
[![Status](https://img.shields.io/badge/Status-Alpha-yellow.svg)]()

---

## What is KPE?

KPE is a parametric geometry engine that completely decouples the **data model** from the **renderer**. You can use the same model in:

- A desktop app (via Tauri)
- A browser (via WASM)
- A headless server for technical drawing generation
- Any 3D renderer: Three.js, Babylon.js, wgpu, Bevy

The core concept: you define an object **once** as a parametric block with rules and constraints, and the engine ensures everything remains consistent when parameters change.

**Current Use Cases:**
- Custom furniture design with CNC cut lists
- Mechanical parts with joints and movement limits (hinges, sliders)
- Any fabricable part where millimeter precision is critical

---

## Design Principles

These decisions are non-negotiable. Any PR violating them will be rejected.

**1. The core is renderer-agnostic.**
`kpe-core` never imports Three.js, wgpu, or any visual library. It exposes raw data.

**2. No file exceeds 300 lines.**
If a module grows, it must be split. Modularity is not optional.

**3. Everything is parametric.**
There is no such thing as a hardcoded value that should be a variable. If it can change, it is a parameter.

**4. Design decisions are documented.**
Every major decision must have its own file in `wiki/decisions/`. If it isn't documented, it doesn't exist. **Note: All documentation within the Wiki must be written in English.**

**5. The schema is the contract.**
Types in `kpe-schema` are the contract between all layers. Changing them is a major decision that requires migration.

---

## Repository Structure

```
kpe/
│
├── README.md                    ← you are here
├── Cargo.toml                   ← Rust workspace
│
├── crates/                      ← System core (Pure Rust)
│   │
│   ├── kpe-schema/              ← Shared types, the contract between layers
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── block.rs         ← BlockDefinition, ParamSchema
│   │   │   ├── geometry.rs      ← GeometryNode, Operation, TransformOp
│   │   │   ├── joint.rs         ← Joint, JointLimits, JointType
│   │   │   ├── constraint.rs    ← Constraint, ConstraintType
│   │   │   ├── material.rs      ← ProceduralMaterial, MaterialLayer
│   │   │   ├── recipe.rs        ← KPERecipe (root document)
│   │   │   └── fabrication.rs   ← CutPiece, CutList, NestingSheet
│   │   └── Cargo.toml
│   │
│   ├── kpe-parametric/          ← Parameter solver and conditional rules
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── solver.rs        ← Solves the dependency graph
│   │   │   ├── expression.rs    ← Math expression evaluator
│   │   │   ├── condition.rs     ← Boolean condition evaluator
│   │   │   ├── rule_engine.rs   ← Applies conditional rules to the recipe
│   │   │   └── catalog.rs       ← Block catalog (custom + industry standards)
│   │   └── Cargo.toml
│   │
│   ├── kpe-geometry/            ← High-level geometric operations
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── brep.rs          ← B-Rep half-edge skeleton (faces, edges, vertices)
│   │   │   ├── csg.rs           ← Boolean operations (Union / Subtract / Intersect)
│   │   │   ├── predicates.rs    ← orient2d/orient3d exact geometric predicates
│   │   │   ├── intersection.rs  ← Triangle-triangle intersection (Möller 1997)
│   │   │   ├── bvh.rs           ← AABB BVH with SAH for spatial acceleration
│   │   │   ├── classify.rs      ← Ray-casting winding number inside/outside test
│   │   │   ├── stitch.rs        ← Vertex welding and manifold stitching
│   │   │   ├── transform.rs     ← Transformation matrices, non-baked
│   │   │   ├── joint.rs         ← Joint movement logic
│   │   │   ├── mesh.rs          ← Triangulation for rendering
│   │   │   └── sketch.rs        ← 2D profiles for extrusion/revolving
│   │   └── Cargo.toml
│   │
│   ├── kpe-fabrication/         ← Cut lists and CNC optimization
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── cutlist.rs       ← Part list generation
│   │   │   ├── nesting.rs       ← Nesting algorithm (bin packing)
│   │   │   ├── grain.rs         ← Wood grain constraints
│   │   │   ├── dxf.rs           ← DXF export
│   │   │   └── svg.rs           ← Technical drawing SVG export
│   │   └── Cargo.toml
│   │
│   ├── kpe-material/            ← Procedural materials
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── generator.rs     ← Generates procedural textures (noise, wood, etc.)
│   │   │   ├── uv.rs            ← World-scale UV mapping
│   │   │   ├── instance_vars.rs ← Per-instance variables (seed, text, batch)
│   │   │   └── text_overlay.rs  ← Dynamic text on surfaces
│   │   └── Cargo.toml
│   │
│   └── kpe-wasm/                ← WASM bindings for JavaScript/TypeScript
│       ├── src/
│       │   ├── lib.rs
│       │   ├── api.rs           ← Functions exposed to JS (wasm-bindgen)
│       │   └── convert.rs       ← Conversion between Rust and JS types
│       └── Cargo.toml
│
├── apps/                        ← Applications consuming the crates
│   │
│   ├── desktop/                 ← Tauri app (Native desktop)
│   │   ├── src-tauri/
│   │   │   ├── src/
│   │   │   │   ├── main.rs
│   │   │   │   └── commands.rs  ← Tauri commands calling kpe-*
│   │   │   └── Cargo.toml
│   │   └── src/                 ← Frontend (React/TypeScript)
│   │       ├── components/
│   │       ├── hooks/
│   │       └── main.tsx
│   │
│   └── web/                     ← Browser app (React + WASM)
│       ├── src/
│       │   ├── components/
│       │   ├── hooks/
│       │   │   └── useKPE.ts    ← Hook wrapping the WASM module
│       │   └── main.tsx
│       └── package.json
│
└── wiki/                        ← Project documentation (MUST BE IN ENGLISH)
    ├── decisions/               ← ADRs (Architecture Decision Records)
    │   ├── 000-template.md
    │   ├── 001-rust-as-core-language.md
    │   ├── 002-manifold-for-csg.md
    │   ├── 003-004-schema-and-transforms.md
    │   └── 005-mesh-csg-pipeline.md
    ├── diagrams/                ← Architecture diagrams
    │   ├── architecture.md
    │   ├── data-flow.md
    │   └── parametric-solver.md
    └── scripts/                 ← Python scripts for code navigation
        ├── find_type.py         ← Locate type definitions
        ├── find_usages.py       ← Locate function usages
        ├── check_modularity.py  ← Detect files exceeding 300 lines
        ├── doc_coverage.py      ← Verify docstrings for all pub items
        └── dep_graph.py         ← Generate dependency graph between crates
```

---

## Quickstart

### Prerequisites

```bash
# Rust (1.75 or higher)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# WASM Target
rustup target add wasm32-unknown-unknown

# wasm-pack (to compile to WASM)
cargo install wasm-pack

# Node.js 18+ (for web/desktop apps)
# Tauri CLI
cargo install tauri-cli
```

### Clone and Compile Core

```bash
git clone https://github.com/jvchiappini/KineticParametricEngine.git
cd KineticParametricEngine

# Compile all crates
cargo build

# Run tests
cargo test

# Compile to WASM
cd crates/kpe-wasm
wasm-pack build --target web
```

### Run Desktop App

```bash
cd apps/desktop
npm install
cargo tauri dev
```

### Run Web App

```bash
cd apps/web
npm install
npm run dev
```

---

## How it Works — Data Flow

```
User defines a KPERecipe (JSON)
            │
            ▼
    kpe-parametric
    ├── Evaluates variables and expressions
    ├── Applies conditional rules
    │   ("if width > 800 → add reinforcement")
    └── Produces a RecipeResolved
            │
            ▼
    kpe-geometry
    ├── Builds scene tree with matrices (non-baked)
    ├── Applies CSG operations (custom pipeline: BVH → Möller → Classify → Stitch)
    ├── Resolves joints and constraints
    └── Produces GeometryOutput { brep, mesh, world_matrices }
            │
        ┌───┴───────────────────────────────┐
        │                                   │
        ▼                                   ▼
  kpe-fabrication                    (External Renderer)
  ├── CutList                        Three.js / wgpu /
  ├── Optimized Nesting              Babylon.js / Canvas2D
  └── DXF / SVG Export              Consumes mesh + matrices
```

The renderer **never** modifies the model. It only consumes `GeometryOutput` and draws it. The entire state lives in `KPERecipe`.

---

## Parametric Blocks — Core Concept

A block is the fundamental unit of KPE. It is equivalent to an AutoCAD Dynamic Block but with full conditional logic.

```json
{
  "id": "side_panel",
  "label": "Side Panel",

  "params": {
    "width":     { "type": "number", "default": 600,  "min": 200,  "max": 1200, "unit": "mm" },
    "height":    { "type": "number", "default": 2100, "min": 500,  "max": 3000, "unit": "mm" },
    "thickness": { "type": "number", "default": 18,   "min": 9,    "max": 36,   "unit": "mm" },
    "material":  { "type": "enum",   "default": "mdf", "options": ["mdf", "plywood", "solid_wood"] },
    "has_holes": { "type": "boolean","default": true }
  },

  "variables": {
    "inner_width": "params.width - 2 * params.thickness"
  },

  "rules": [
    {
      "when": "params.width > 800",
      "then": [
        { "add_child": { "id": "center_reinforcement", "type": "box",
          "params": { "width": "params.thickness", "height": "params.height * 0.6",
                      "depth": "params.thickness" } } }
      ]
    },
    {
      "when": "params.has_holes",
      "then": [
        { "add_operation": {
          "type": "subtract",
          "tool": { "type": "cylinder", "params": { "radius": 16, "height": "params.thickness + 2" } },
          "array": { "type": "linear", "axis": "Y", "count": "Math.floor(params.height / 32)", "spacing": 32 }
        }}
      ]
    }
  ],

  "joints": {
    "hinge_top": {
      "type": "revolute",
      "axis": [0, 1, 0],
      "limits": { "min": 0, "max": "params.max_open_angle", "damping": 0.1 }
    }
  }
}
```

---

## Procedural Materials

Materials in KPE are **not static textures**. They are generated based on the actual size of the object and can have per-instance variables.

```json
{
  "base": { "color": "#8B5E3C", "roughness": 0.8, "metalness": 0 },
  "uv_mode": "world_scale",
  "uv_scale": [1, 1],

  "instance_vars": {
    "seed":     { "type": "random_int", "range": [0, 9999] },
    "label":    { "type": "string",     "default": "" },
    "batch_id": { "type": "string" }
  },

  "layers": [
    {
      "type": "wood_grain",
      "seed": "instance.seed",
      "direction": "params.grain_direction"
    },
    {
      "type": "text_overlay",
      "content": "instance.label",
      "position": [10, 10],
      "font_size": 8,
      "visible": "instance.label !== ''"
    }
  ]
}
```

Each piece on the same board can have its own unique grain (`instance.seed`) and its own identification label for manufacturing.

---

## CNC Fabrication

KPE automatically generates:

- **CutList**: A list of all parts with dimensions, quantity, material, and grain direction.
- **Nesting**: Optimization of part layout on the board to minimize waste, respecting wood grain constraints.
- **DXF**: CNC-ready cutting plans.
- **SVG**: Presentation plans for the client.

```rust
use kpe_fabrication::{generate_cutlist, NestingConfig, SheetSize};

let cutlist = generate_cutlist(&resolved_recipe)?;

let nesting = cutlist.optimize_nesting(NestingConfig {
    sheet: SheetSize { width: 2440.0, height: 1220.0 },
    blade_width: 3.2,
    respect_grain: true,
    margin: 10.0,
})?;

nesting.export_dxf("cutting_plans.dxf")?;
nesting.export_svg("client_plans.svg")?;
```

---

## Coding Conventions

### Rust

```rust
/// Single-line module description.
///
/// Longer explanation if necessary. Always describe
/// what the module does and what it does NOT do (boundaries).
///
/// # Examples
///
/// ```rust
/// use kpe_parametric::solver::Solver;
/// let solver = Solver::new();
/// ```
pub mod solver;
```

```rust
/// Resolves the parameter dependency graph.
///
/// Evaluates expressions in topological order so that every
/// variable only uses already-resolved values.
///
/// # Errors
///
/// Returns [`SolverError::CircularDependency`] if the graph contains
/// a dependency cycle.
///
/// # Examples
///
/// ```rust
/// let recipe = KPERecipe::default();
/// let resolved = solver.resolve(&recipe)?;
/// assert_eq!(resolved.variables["inner_width"], 564.0);
/// ```
pub fn resolve(&self, recipe: &KPERecipe) -> Result<ResolvedRecipe, SolverError> {
    // max 50 lines per function
    // if it grows, extract into private functions with descriptive names
}
```

**Strict Rules:**
- Every `pub` item (struct, fn, enum, trait) **must** have a docstring.
- Functions must be 50 lines max. If they grow, split them.
- Files must be 300 lines max. If they grow, split into modules.
- No `unwrap()` in production code. All errors must propagate with `?`.
- No unnecessary `clone()`. If cloning a large structure, comment why.

### Commits

```
type(crate): short description in present tense

feat(kpe-parametric): add boolean condition evaluator
fix(kpe-geometry): fix pivot transform in world space  
docs(wiki): add ADR-005 regarding nesting algorithm
test(kpe-fabrication): add wood grain constraint tests
refactor(kpe-schema): separate joint.rs from geometry.rs
```

Types: `feat` | `fix` | `docs` | `test` | `refactor` | `chore`

---

## Navigation Scripts (wiki/scripts/)

The repository includes Python scripts to navigate and audit the codebase:

```bash
# Find where a type is defined
python wiki/scripts/find_type.py KPERecipe

# Find all usages of a function
python wiki/scripts/find_usages.py resolve_expression

# Detect files exceeding 300 lines (modularity violation)
python wiki/scripts/check_modularity.py

# Verify docstring coverage on public items
python wiki/scripts/doc_coverage.py

# Generate crate dependency graph (outputs diagrams/deps.dot)
python wiki/scripts/dep_graph.py
```

---

## Decision Making (ADRs)

All significant architecture decisions are documented in `wiki/decisions/` following the ADR (Architecture Decision Record) format. **Reminder: All ADRs and Wiki entries must be written in English.**

See [`wiki/decisions/000-template.md`](wiki/decisions/000-template.md) for the format.

Documented decisions:
- [ADR-001](wiki/decisions/001-rust-as-core-language.md) — Rust as core language
- [ADR-002](wiki/decisions/002-manifold-for-csg.md) — Original CSG plan (superseded by ADR-005 for implementation order)
- [ADR-003](wiki/decisions/003-004-schema-and-transforms.md) — kpe-schema as the contract between layers
- [ADR-004](wiki/decisions/003-004-schema-and-transforms.md) — Matrices for transforms, not baked into vertices
- [ADR-005](wiki/decisions/005-mesh-csg-pipeline.md) — Mesh-based CSG pipeline (iterative, not B-Rep first)

---

## Roadmap

### Phase 1 — Parametric Core ✅
- [x] `kpe-schema`: Complete base types
- [x] `kpe-parametric`: Expression and condition evaluator
- [x] `kpe-parametric`: Rule engine (conditional logic)
- [x] `kpe-geometry`: Basic primitives (box, cylinder, sphere)
- [x] `kpe-geometry`: CSG pipeline (BVH + Möller + Classify + Stitch)
- [ ] `kpe-wasm`: Basic browser bindings
- [ ] Unit tests across all crates
- [ ] Triangle splitting for exact CSG boundaries

### Phase 2 — Fabrication
- [ ] `kpe-fabrication`: Cut list generation
- [ ] `kpe-fabrication`: Basic nesting (guillotine cuts)
- [ ] `kpe-fabrication`: Wood grain constraints
- [ ] `kpe-fabrication`: DXF and SVG export

### Phase 3 — Joints and Motion
- [ ] `kpe-geometry`: Joints (revolute, prismatic)
- [ ] `kpe-geometry`: Physical limits and soft stops
- [ ] `kpe-geometry`: Constraints between nodes

### Phase 4 — Procedural Materials
- [ ] `kpe-material`: Generators (wood, concrete, noise)
- [ ] `kpe-material`: World-scale UVs
- [ ] `kpe-material`: Per-instance variables
- [ ] `kpe-material`: Text overlay

### Phase 5 — Apps
- [ ] `apps/web`: Basic Three.js viewer
- [ ] `apps/web`: Parameter editor
- [ ] `apps/desktop`: Tauri app
- [ ] 2D Sketch → 3D Extrusion

---

## Component Catalog

KPE supports two block sources:

**Custom Catalog** (`catalog/custom/`): User-defined blocks in JSON following the `kpe-schema`.

**Industry Standards** (`catalog/industry/`): Pre-defined blocks for industrial components.

```
catalog/
├── custom/          ← Your own blocks
│   ├── furniture/
│   └── parts/
└── industry/        ← Industrial standards
    ├── blum/        ← Blum slides and hinges
    ├── hafele/      ← Häfele fittings
    └── hardware/    ← Standard screws and bolts
```

Every catalog block is a `.kpe.json` file that follows the exact same schema as any other block. There is no special format for industrial standards.

---

## Contributing

PRs are welcome. Before submitting:

1. Every schema change requires an approved ADR.
2. All new code requires tests.
3. Every `pub` item requires a docstring.
4. `check_modularity.py` must pass (no files > 300 lines).
5. `doc_coverage.py` must pass (100% of pub items documented).
6. All Wiki documentation must be written in English.

Open an issue first if you're planning significant changes — especially
to `kpe-schema` or the CSG pipeline.

---

## License

MIT — do whatever you want, with attribution.

---

*KPE is in alpha. The schema may change without notice until reaching v1.0.*