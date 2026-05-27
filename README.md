# KPE — Kinetic Parametric Engine

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://rustup.rs)
[![WASM](https://img.shields.io/badge/Target-WASM%20%2B%20Native-green.svg)]()
[![Status](https://img.shields.io/badge/Status-Alpha-yellow.svg)]()

A general-purpose parametric CAD engine with a Rust core and a browser-based UI. Designed for precise geometry, dynamic blocks with conditional logic, and CNC-ready fabrication output.

## Quickstart

```bash
git clone https://github.com/jvchiappini/KineticParametricEngine.git
cd KineticParametricEngine

cargo build                           # compile all Rust crates
cargo test                            # run tests

cd apps/web
npm install && npm run dev            # run the web UI (React + Three.js + WASM)
```

### WASM build (required for the web app)

```bash
wasm-pack build crates/kpe-wasm --target web --out-dir ../../apps/web/kpe-wasm
```

## Architecture

All computation lives in **Rust** (`crates/`): geometry, constraint solver, CSG, extrusion, export.  
The **frontend** (`apps/web/`, TypeScript + React + Three.js) is pure UI — canvas rendering, interaction, and panels.

Communication is via JSON serialization through WASM. The constraint solver runs natively in Rust (iterative gradient descent). A command pattern in the frontend handles undo/redo.

**Key constraint:** no file exceeds 700 lines. Every module has a single responsibility.

## Repository Layout

```
crates/          ← Rust workspace (core engine)
  kpe-schema/      Shared types — the contract between all layers
  kpe-geometry/    Geometry ops, constraints, CSG, extrusion, 2D sketch
  kpe-parametric/  Parameter solver, rules engine, catalog
  kpe-fabrication/ Cut lists, nesting, DXF/SVG export
  kpe-material/    Procedural texture generation
  kpe-wasm/        WASM bindings exposed to JS

apps/            ← Applications
  web/             Browser UI (React + Three.js + WASM)
  desktop/         Tauri desktop app (inactive)

wiki/            ← Documentation (all in English)
  decisions/       Architecture Decision Records
  roadmaps/        Development roadmap
  diagrams/        Architecture and data-flow diagrams
  scripts/         Utility scripts (modularity check, doc coverage, etc.)
```

## Documentation

All detailed documentation lives in the [`wiki/`](wiki/) directory:

- [Architecture & data flow](wiki/diagrams/architecture.md)
- [Roadmap](wiki/roadmaps/KPE-roadmap.md)
- [Architecture Decision Records](wiki/decisions/)
- [Coding conventions & PR process](CONTRIBUTING.md)

## What KPE can do

- 2D sketch with points, lines, arcs, circles, polylines
- 13 constraint types (horizontal, vertical, coincident, parallel, etc.)
- Iterative gradient-descent constraint solver
- Extrusion with bevel/chamfer
- 3D viewport with transform controls
- Save/load sketches (`.kpe` format)
- Parametric blocks with conditional rules (via `kpe-parametric`)
- CSG boolean operations (union, subtract, intersect)
- Cut list generation and nesting (via `kpe-fabrication`)

## Project Status

Alpha. The schema may change without notice until v1.0. The current focus is the 2D sketch → 3D extrusion pipeline in the web UI.

## License

MIT — see [LICENSE](LICENSE).
