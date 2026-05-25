# ADR-001 — Rust as the Core Language

**Status:** `accepted`
**Date:** 2025-05-25
**Author:** KPE Project

---

## Context

KPE requires a geometric core that can run in three different contexts without being rewritten:

1.  **Browser** — for the web application.
2.  **Native Desktop** — for the Tauri application with maximum performance.
3.  **Headless Server** — for batch drawing generation (future requirement).

The core handles millimeter-precision mathematical operations, scene trees with potentially thousands of nodes, and optimization algorithms (nesting). Performance is critical.

The primary constraint: we want to avoid rewriting the core when moving from web to desktop. A single codebase for all targets.

## Options Considered

### Option A — Rust

Compiles to native WASM via `wasm-bindgen` and to native binaries for desktop. The same source code serves both targets. It has a growing CAD ecosystem (`truck`, `manifold-rs`) and memory safety guaranteed by the compiler.

**Pros:**
- Single codebase for WASM + Native + Server.
- Maximum performance across all targets.
- Garbage-collector-free (predictable performance for heavy geometric operations).
- Mature mathematical crate ecosystem (`nalgebra`, `glam`).
- The compiler eliminates entire classes of bugs (null pointers, data races, use-after-free).

**Cons:**
- Steep learning curve (the borrow checker).
- Slower iteration speed compared to pure TypeScript.
- CAD ecosystem is less mature than C++.

### Option B — Pure TypeScript (+ external manifold WASM)

Use TypeScript for everything and consume Manifold as a precompiled WASM module.

**Pros:**
- Very fast iteration.
- Single language for both frontend and core.
- Easy to debug in the browser.

**Cons:**
- Desktop would require Electron (resource-heavy) or a rewrite to Rust eventually.
- No performance guarantees for heavy operations.
- As the project grows, a core written in TS becomes significant technical debt.

### Option C — C++

The language of industrial CAD software. OpenCASCADE is written in C++.

**Pros:**
- The most mature CAD ecosystem in the world.
- Maximum possible performance.

**Cons:**
- WASM implementation is much more complex (Emscripten has limitations).
- Manual memory management (prone to difficult-to-trace bugs).
- Developer experience (DevEx) is significantly worse than Rust.

## Decision

**Option A (Rust) was chosen.**

The deciding factor was the compilation story: a single codebase that compiles to WASM for the browser and native binaries for the desktop. This eliminates the need for a future rewrite, which was the most important requirement.

While the learning curve for Rust is significant, the author already has basic experience with it. Furthermore, the errors the borrow checker prevents are exactly the types of bugs that frequently appear in complex geometry (e.g., invalid references to scene tree nodes, concurrent mutations).

## Consequences

**Positive:**
- The core is written once and works on all targets.
- Maximum performance for nesting and CSG operations.
- The compiler acts as a safety net for complex mathematical code.

**Negative / Trade-offs:**
- Initial iteration is slower than TypeScript.
- Some algorithms (such as dynamic expressions) are more verbose in Rust than in JS.

**Neutral / Points to Consider:**
- WASM bindings are generated using `wasm-bindgen` and `wasm-pack`.
- The `kpe-wasm` crate is the only one aware of `wasm-bindgen` — the rest are pure Rust.
- Web technologies (HTML/CSS/React) will always be used for the UI, rather than Rust-native UI libraries.

## References

- [wasm-bindgen guide](https://rustwasm.github.io/docs/wasm-bindgen/)
- [Tauri — Rust + WebView](https://tauri.app)
- [truck — B-Rep in Rust](https://github.com/ricosjp/truck)