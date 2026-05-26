# Contributing to KPE

First off, thank you for considering contributing. Every PR, issue, and discussion
makes this project better.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Design Principles](#design-principles)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Coding Conventions](#coding-conventions)
- [Testing](#testing)
- [Benchmarks](#benchmarks)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)
- [ADR Process](#adr-process)
- [Questions?](#questions)

## Code of Conduct

This project and everyone participating in it is governed by the
[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). By participating, you are expected
to uphold this code.

## Design Principles

Read [README.md](README.md#design-principles) first. These are non-negotiable:

1. **The core is renderer-agnostic.** — No visual library in `kpe-*` crates.
2. **No file exceeds 300 lines.** — Split or refactor if it grows.
3. **Everything is parametric.** — No hardcoded values that should be variables.
4. **Design decisions are documented.** — Every ADR in `wiki/decisions/`.
5. **The schema is the contract.** — `kpe-schema` changes require migration.

## Getting Started

### Prerequisites

```bash
# Rust 1.75+
rustup install 1.75.0

# WASM target for browser builds
rustup target add wasm32-unknown-unknown

# wasm-pack
cargo install wasm-pack

# Node.js 18+ for apps
```

### Clone and Build

```bash
git clone https://github.com/jvchiappini/KineticParametricEngine.git
cd KineticParametricEngine

# Compile all Rust crates
cargo build

# Run all tests
cargo test --all

# Run clippy
cargo clippy --all -- -D warnings
```

### WASM Build

```bash
cd crates/kpe-wasm
wasm-pack build --target web
```

### Web App

```bash
cd apps/web
npm install
npm run dev
```

## Development Workflow

1. **Pick an issue** — or open one first if your change is significant.
2. **Create a branch** — `feat/description`, `fix/description`, `docs/description`.
3. **Make changes** — follow the conventions below.
4. **Run checks** — `cargo test`, `cargo clippy`, `python wiki/scripts/check_modularity.py`.
5. **Open a PR** — link the issue, describe the change.

### Commit Messages

```
type(crate): short description in present tense

feat(kpe-geometry): add push-pull extrude for face selection
fix(kpe-parametric): resolve circular dependency panic
docs(wiki): add ADR-006 for sketch constraint system
test(kpe-fabrication): add nesting test with grain constraints
refactor(kpe-schema): split geometry.rs into submodules
```

Types: `feat` | `fix` | `docs` | `test` | `refactor` | `chore`

## Coding Conventions

### Rust

- Every `pub` item **must** have a docstring (`///`).
- Functions: max **50 lines**. Split into private helpers if longer.
- Files: max **300 lines**. Split into modules if longer.
- No `unwrap()` in production code. Use `?` and `thiserror`.
- No unnecessary `clone()`. If cloning a large structure, comment why.
- Use `glam` for vector/matrix math (not `nalgebra` in new code unless required).
- Prefer `thiserror` for error types.

```rust
/// Resolves the parameter dependency graph.
///
/// Evaluates expressions in topological order.
///
/// # Errors
///
/// Returns `SolverError::CircularDependency` on cycle detection.
pub fn resolve(&self, recipe: &KPERecipe) -> Result<ResolvedRecipe, SolverError> {
    // max 50 lines
}
```

### TypeScript / JavaScript (apps)

- Use TypeScript with strict mode.
- No `any` unless absolutely necessary.
- Prefer `const` over `let`.
- Use Three.js idioms when working with 3D.

## Testing

- **Unit tests**: inline `#[cfg(test)] mod tests { ... }` in each module.
- **Integration tests**: add to `tests/` in the relevant crate.
- **Test data**: place JSON fixtures in `test_data/`.
- Run: `cargo test --all`

All new features require tests. Bug fixes require a regression test.

## Benchmarks

Benchmarks use [criterion](https://github.com/bheisler/criterion.rs).

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark group
cargo bench --bench csg
cargo bench --bench sketch
```

Add new benchmarks in the relevant crate's `benches/` directory.
Benchmark results are stored in `target/criterion/` — do not commit.

## Documentation

- All wiki content **must be in English**.
- Architecture decisions go in `wiki/decisions/` following the ADR template.
- Docstrings on all `pub` items are mandatory.
- Run `python wiki/scripts/doc_coverage.py` to verify coverage.

## Pull Request Process

1. Ensure all checks pass:
   - `cargo test --all`
   - `cargo clippy --all -- -D warnings`
   - `python wiki/scripts/check_modularity.py`
   - `python wiki/scripts/doc_coverage.py`
2. Update the README.md or wiki docs if your change affects public API or workflow.
3. Schema changes require an approved ADR first.
4. A maintainer will review your PR. Address feedback promptly.
5. Squash merge when approved.

### PR Checklist

- [ ] Tests added/updated
- [ ] Documentation updated (docstrings + wiki if applicable)
- [ ] Modularity check passes (no files > 300 lines)
- [ ] Docstring coverage 100% on pub items
- [ ] No `unwrap()` in production code
- [ ] Branch is up to date with main

## ADR Process

Significant architecture decisions (schema changes, new crates, pipeline changes)
require an Architecture Decision Record:

1. Copy `wiki/decisions/000-template.md` to `wiki/decisions/NNN-description.md`.
2. Fill out the template: context, decision, consequences.
3. Open a PR with the ADR and any implementation changes.
4. ADR is approved through PR review consensus.

## Questions?

- Open a [Discussion](https://github.com/jvchiappini/KineticParametricEngine/discussions)
- Join the Discord (link TBD)
- Tag `@jvchiappini` in issues

---

*KPE is in alpha. The schema may change without notice until v1.0.*
