# KPE Roadmap — Superar a FreeCAD, diseñar muebles

> **Arquitectura (ACTUALIZADA 2026-05-27):** Rust (`crates/`) es el motor — geometría, solver, constraints, CSG, exportación.
> **Pivote Estratégico:** El frontend de TypeScript/React/WASM ha sido DEPRECADO a favor de una **Aplicación Nativa de Escritorio (`apps/desktop/`)**. Se usará **Bevy + egui** para maximizar rendimiento CAD, eliminar el puente WASM JS, y enfocarnos 100% en el core de Rust y shaders de GPU.
> Toda feature nueva se implementa **primero en Rust Puro** de forma agnóstica (`kpe-geometry` / `kpe-schema`) y luego consumida por la Vista de Bevy.

**Leyenda:** `[ ]` por hacer · `[x]` completado · `[-]` en progreso  
**Tags:** `[core]` = lógica pura (headless) · `[ui]` = Interfaz/Render (Egui/Bevy) · `[both]` = ambos lados

---

## Fase 0 — Pivote a Desktop Nativo (Completado)

> El objetivo principal de esta fase fue eliminar el límite de WASM y navegadores para obtener todo el poder del S.O.
> El motor ahora levanta en Bevy 0.15 usando MSVC linker con soporte para SSAO y Manifold-CSG C++.

- `[x]` `[core]` Integrar C++ Manifold como Kernel CSG robusto y oficial por defecto.
- `[x]` `[ui]` Mudar el render de Three.js a Bevy + bevy_egui.
- `[x]` `[both]` Establecer el puente reactivo (UI → State → Geometría CSG → Mesh → GPU Bevy) en tiempo real (`< 5ms` por operación).
- `[x]` `[ui]` Ambient occlusion (SSAO) nativo para visibilidad fotorealista profunda.
- `[x]` `[core]` Desvincular cualquier dependencia de la capa de render en el solver o crates.

## Fase 1 — UX Profesional (Alta prioridad)

- `[ ]` `[ts]` Undo/Redo UI — atajos Ctrl+Z/Y + botones (history commands en Rust)
- `[ ]` `[rust]` `[ts]` Edición paramétrica — doble-click en cota → Rust re-suelve con nuevo valor
- `[ ]` `[rust]` `[ts]` Measure tool — Rust calcula distancias/ángulos, TS muestra overlay
- `[ ]` `[ts]` Selección avanzada — caja/lasso, select all, filtro por tipo
- `[ ]` `[rust]` `[ts]` Copy/Cut/Paste entidades — Rust duplica entidades en el documento
- `[ ]` `[rust]` Mejorar solver 2D — Gauss-Seidel con damping, soporte para restricciones cíclicas

## Fase 2 — Modelado de Muebles (Alta prioridad)

- `[ ]` `[rust]` Duplicate / Array tool — Rust genera N copias con transformaciones
- `[ ]` `[rust]` Mirror / Symmetry — sobre eje arbitrario, resuelto en Rust
- `[ ]` `[rust]` Fillet / Chamfer 3D — operaciones sobre mallas en `kpe-geometry`
- `[ ]` `[rust]` Extrude con taper angle — `kpe-geometry/src/extrude.rs`
- `[ ]` `[rust]` Boolean operations — unir/restar/intersección (CSG ya existe en Rust)
- `[ ]]` `[ts]` Picker de materiales por sketch — color, textura procedural vía `kpe-material`

## Fase 3 — Datos y Exportación (Media prioridad)

- `[ ]` `[rust]` Exportar STL — desde `TriangleMesh` (binario y ASCII)
- `[ ]` `[rust]` Exportar OBJ + MTL — vértices, normales, UVs, materiales
- `[ ]]` `[rust]` Exportar STEP — ya hay esqueleto en `kpe-cli/src/export/step.rs`
- `[ ]` `[rust]` Save/Load `.kpe` — `KPERecipe` completo serializado a JSON/BSON
- `[ ]` `[rust]` `[ts]` Material preview — `kpe-material` genera texturas, Three.js las renderiza

## Fase 4 — Construcción (Media prioridad)

- `[ ]` `[rust]` Assembly system — `kpe-geometry/src/joint.rs` + transform tree
- `[ ]` `[rust]` Fasteners library — tornillos, tuercas, bisagras como bloques paramétricos
- `[ ]` `[ts]` Importar imagen de referencia — overlay en el canvas 2D para calcar
- `[ ]` `[rust]` `[ts]` Grid configurable — tamaño, snap, sub-divisiones (Rust calcula snap points)
- `[ ]` `[rust]` `[ts]` Layer system — Rust filtra entidades por capa, TS renderiza

## Fase 5 — Poder CAD (Baja prioridad)

- `[ ]` `[rust]` Sketch on face — crear sketch sobre cara existente (Rust calcula plano local)
- `[ ]` `[rust]` Pocket / Hole — taladros paramétricos (CSG subtract ya existe)
- `[ ]` `[rust]` Revolve / Lathe — `kpe-geometry` `RevolveDef` ya está en schema
- `[ ]` `[rust]` Sweep — recorrer perfil a lo largo de camino 3D
- `[ ]` `[rust]` Loft — transición entre perfiles (malla interpolada)
- `[ ]` `[rust]` `[ts]` Ecuaciones y variables — `ancho = 2 * alto` via `kpe-parametric`
- `[ ]` `[rust]` Part library — catálogo de componentes paramétricos (`kpe-parametric/src/catalog.rs`)

## Fase 6 — Nicho / Diferenciador

- `[ ]` `[ts]` Modo offline (PWA) — service worker, cache del WASM + assets
- `[ ]` `[rust]` `[ts]` Compartir sketch por URL — serializar `KPERecipe` en hash/query string
- `[ ]` `[rust]` `[ts]` Real-time collaboration — OT/CRDT sobre operaciones del documento Rust
- `[ ]` `[rust]` AI sugerencias — detecta intención basado en geometría, sugiere constraints/herramientas
- `[ ]` `[rust]` Plugins / Scripting — API WASM para que terceros extiendan herramientas
