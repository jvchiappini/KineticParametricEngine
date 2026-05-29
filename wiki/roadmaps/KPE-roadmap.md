# KPE Roadmap — Superar a FreeCAD, diseñar muebles

> **Arquitectura:** 100% Desktop Native. Motor Rust (`crates/`) + UI Bevy + egui (`apps/desktop/`).
> Web app (`apps/web/`) deprecada — no se mantiene más.
>
> **Leyenda:** `[ ]` por hacer · `[x]` completado · `[-]` en progreso
> **Tags:** `[core]` = lógica pura (headless) · `[ui]` = Interfaz/Render (Egui/Bevy) · `[both]` = ambos lados

---

## Fase 0 — Fundación (Completado)

- `[x]` `[core]` Integrar C++ Manifold como Kernel CSG robusto y oficial por defecto
- `[x]` `[ui]` Bevy + egui: orbit camera, gizmos, SSAO, paneles (toolbar, scene tree, properties, status bar)
- `[x]` `[both]` Puente reactivo (UI → State → Geometría CSG → Mesh → GPU Bevy) con MeshCache
- `[x]` `[ui]` SSAO nativo para visibilidad fotorealista profunda
- `[x]` `[core]` Desvincular cualquier dependencia de render en el solver o crates
- `[x]` `[both]` Save/Load `.kpe` — `KPERecipe` serializado a JSON con diálogo nativo
- `[x]` `[core]` Exportar STL, STEP, OBJ (parcial MTL)

## Fase 1 — UX Profesional (Completado)

- `[x]` `[ui]` Undo/Redo — atajos Ctrl+Z/Y en Bevy + botones en toolbar
- `[x]` `[ui]` Edición paramétrica — doble-click en cota → re-solver con nuevo valor
- `[x]` `[core]` Measure tool — Rust calcula distancias/ángulos, UI muestra overlay
- `[x]` `[ui]` Selección avanzada — Ctrl+A select all en sketch y 3D
- `[x]` `[core]` Copy/Cut/Paste entidades — duplicar entidades en el documento
- `[x]` `[core]` Mejorar solver 2D — batch gradient descent con cyclic constraints + damping
- `[x]` `[core]` Undo/Redo para sketch editor (operaciones de dibujo)

## Fase 2 — Modelado de Muebles (Completado)

- `[x]` `[core]` Boolean operations — unir/restar/intersección (CSG con Manifold + csgrs backends)
- `[x]` `[core]` Revolve / Lathe — `RevolveDef` con eje configurable (X/Y/Z) y segmentos
- `[x]` `[core]` Sweep — perfil a lo largo de camino 3D (Linear, Arc, Helix) con Frenet-frame
- `[x]` `[core]` Ecuaciones y variables — ExpressionEvaluator, RuleEngine, Solver en `kpe-parametric`
- `[x]` `[core]` Duplicate / Array tool — Rust genera N copias con transformaciones
- `[x]` `[core]` Mirror / Symmetry — reflejo sobre plano XY/XZ/YZ vía escala negativa
- `[x]` `[core]` Fillet / Chamfer 3D — modificadores con `smooth_out` Manifold
- `[x]` `[both]` Extrude con taper angle — `taper_angle: Option<f64>` en `ExtrudeDef`, interpolación por rings
- `[x]` `[ui]` Picker de materiales por sketch — `color: Option<String>` en `GeometryNode`, color picker egui + Bevy sync per-node

## Fase 3 — Construcción (Media prioridad)

- `[-]` `[core]` Assembly system — JointEngine (Revolute/Prismatic/Fixed/Ball), falta transform tree completo
- `[-]` `[core]` Part library — Catalog struct existe, faltan componentes paramétricos precargados
- `[ ]` `[core]` Fasteners library — tornillos, tuercas, bisagras como bloques paramétricos
- `[ ]` `[both]` Importar imagen de referencia — overlay en el canvas 2D para calcar
- `[ ]` `[core]` Layer system — Rust filtra entidades por capa, UI renderiza

## Fase 4 — Poder CAD (Baja prioridad)

- `[ ]` `[core]` Sketch on face — crear sketch sobre cara existente (Rust calcula plano local)
- `[ ]` `[core]` Pocket / Hole — taladros paramétricos (CSG subtract ya existe, falta tool)
- `[ ]` `[core]` Loft — transición entre perfiles (malla interpolada)
- `[ ]` `[core]` Material preview — `kpe-material` genera texturas, UI las renderiza

## Fase 5 — Nicho / Diferenciador

- `[ ]` `[core]` Real-time collaboration — OT/CRDT sobre operaciones del documento Rust
- `[ ]` `[core]` AI sugerencias — detecta intención basado en geometría, sugiere constraints/herramientas
- `[ ]` `[core]` Plugins / Scripting — API WASM para que terceros extiendan herramientas
