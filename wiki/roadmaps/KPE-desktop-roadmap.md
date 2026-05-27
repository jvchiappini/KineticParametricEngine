# KPE Desktop App — Roadmap

> **Fecha:** 2026-05-27  
> **Decisiones clave tomadas en esta sesión:**
> - Se abandona la app web (`apps/web` + WASM) como objetivo primario.
> - Se adopta un desktop app nativo en Rust como la aplicación principal.
> - El renderer es una capa intercambiable — el core no depende de ningún renderer específico.
> - Manifold (Google/Blender) reemplaza a csgrs como CSG engine por defecto.

---

## Decisión de Arquitectura: Stack del Desktop

### El Principio Fundamental

```
┌─────────────────────────────────────────────────────────────────┐
│                        KPE Core (Rust)                          │
│                                                                 │
│  kpe-schema  ←  kpe-geometry  ←  kpe-parametric                │
│                      ↓                                          │
│                 TriangleMesh / SketchDef / FeatureTree           │
│                 (datos puros, sin dependencia de renderer)       │
└──────────────────────────┬──────────────────────────────────────┘
                           │  datos puros (vértices, índices, etc.)
              ┌────────────▼────────────┐
              │    Renderer (swappable) │
              │                         │
              │  hoy: Bevy + bevy_egui  │
              │  (ECS solo para vista)  │
              └─────────────────────────┘
```

El core **nunca** importa ningún renderer. Los datos fluyen **unidireccionalmente**:
Core produce → Renderer consume. El renderer es un consumidor pasivo.

### Stack Elegido: Bevy + bevy_egui

| Componente | Librería | Razón |
|---|---|---|
| GPU / 3D render | `bevy` | Motor completo sobre wgpu. Resuelve cámara, luces, materiales, PBR, sobras y windowing out-of-the-box. Acelera el desarrollo drásticamente. |
| UI / Paneles | `egui` (vía `bevy_egui`) | Immediate mode. Zero overhead DOM. Ideal para herramientas CAD. |
| Algebra lineal | `glam` | Ya es la matemática base de Bevy y de kpe-geometry. Integración perfecta. |

**Arquitectura Bevy ECS como Vista:** Usaremos el ECS de Bevy **estrictamente como frontend**. El modelo de datos real vive en KPE Core. Cuando el core genera un `TriangleMesh`, un sistema de Bevy lo convierte a un `bevy::render::mesh::Mesh` y lo renderiza. Todo el estado paramétrico se mantiene fuera del ECS.

---

## Fase 0 — Andamiaje del Desktop App [✔️ COMPLETADO]

> **Objetivo:** Ventana abierta, viewport 3D interactivo con SSAO, cámara orbit, un set de geometrías resueltas booleanamente en tiempo real desde `kpe-geometry` mediante Egui. Todo corriendo.

### F0.1 — Estructura del crate `kpe-app`

```
apps/desktop/
  Cargo.toml           ← kpe-app (binary crate)
  src/
    main.rs            ← event loop, winit setup
    app.rs             ← estado global de la aplicación
    renderer/
      mod.rs           ← trait Renderer (intercambiable)
      wgpu_renderer.rs ← implementación wgpu
      camera.rs        ← arc-ball / orbit camera
      mesh_buffer.rs   ← upload TriangleMesh → GPU buffers
    ui/
      mod.rs           ← egui panels setup
      toolbar.rs       ← herramientas activas
      properties.rs    ← panel de propiedades
      scene_tree.rs    ← árbol de features/objetos
    input.rs           ← mouse, keyboard, drag
```

**Dependencias de `kpe-app`:**

```toml
[dependencies]
kpe-schema    = { path = "../../crates/kpe-schema" }
kpe-geometry  = { path = "../../crates/kpe-geometry" }  # Manifold default
kpe-parametric = { path = "../../crates/kpe-parametric" }
wgpu          = "22"
egui          = "0.31"
egui-wgpu     = "0.31"
egui-winit    = "0.31"
winit         = "0.30"
glam          = { workspace = true }
pollster      = "0.4"   # block_on para async wgpu
bytemuck      = "1"     # cast de structs a bytes para GPU
```

### F0.2 — Trait `Renderer` (intercambiable)

```rust
// apps/desktop/src/renderer/mod.rs
pub trait Renderer {
    fn upload_mesh(&mut self, id: &str, mesh: &TriangleMesh);
    fn remove_mesh(&mut self, id: &str);
    fn set_camera(&mut self, camera: &Camera);
    fn render_frame(&mut self, ui_output: egui::FullOutput);
    fn resize(&mut self, width: u32, height: u32);
}
```

El resto de la app solo habla con este trait. Cambiar de wgpu a Bevy a futuro = implementar el trait nuevo.

### F0.3 — Pipeline de render mínimo

**Shaders WGSL:**
```
vertex shader:  posición + normal → NDC (model * view * projection)
fragment shader: Lambert shading con 2 luces fijas (key + fill)
```

**Buffers mínimos:**
- `VertexBuffer`: `[f32; 3]` posición + `[f32; 3]` normal  
- `IndexBuffer`: `u32` índices de triángulos  
- `UniformBuffer`: `mat4` MVP + `vec3` color base

**Primer render:** un `Box` generado por `Manifold::cube(1.0, 1.0, 1.0, true)` → convertido a `TriangleMesh` → subido a GPU → renderizado.

### F0.4 — Cámara Orbit (Arc-Ball)

```rust
pub struct OrbitCamera {
    pub target:   glam::Vec3,   // punto central
    pub yaw:      f32,          // rotación horizontal
    pub pitch:    f32,          // rotación vertical (clamped ±89°)
    pub distance: f32,          // distancia al target
    pub fov_y:    f32,          // field of view vertical
}
```

Controles:
- **Botón derecho + drag** → orbita
- **Rueda** → zoom (ajusta `distance`)
- **Botón medio + drag** → pan (ajusta `target`)
- **F** → fit all (ajusta `distance` al bounding box de la escena)

---

## Fase 1 — UI Base

> **Objetivo:** Layout profesional. El usuario puede ver y manipular propiedades básicas.

### Layout Principal

```
┌─────────────────────────────────────────────────────────────────┐
│  Toolbar  [New] [Open] [Save] [Undo] [Redo] | [Herramientas]   │
├────────────┬────────────────────────────────┬───────────────────┤
│            │                                │                   │
│  Scene     │                                │  Properties       │
│  Tree      │      Viewport 3D (wgpu)        │  Panel            │
│            │                                │                   │
│  ▼ Doc     │                                │  ▼ Extrude_001    │
│    Sketch  │                                │    Distance: 45mm │
│    Extrude │                                │    Cap: ✓         │
│    ...     │                                │                   │
│            │                                │                   │
├────────────┴────────────────────────────────┴───────────────────┤
│  Status bar: [x: 0.0  y: 0.0  z: 0.0]  [Tris: 1,248]  [60fps] │
└─────────────────────────────────────────────────────────────────┘
```

### F1.1 — Scene Tree Panel
- Árbol jerárquico de objetos/features del documento.
- Click → selecciona y resalta en viewport.
- Ícono de visibilidad toggle (👁).

### F1.2 — Properties Panel
- Muestra propiedades del objeto seleccionado.
- Campos editables: float, int, bool, enum.
- Cambio → dispara re-evaluación en `kpe-geometry`.

### F1.3 — Status Bar
- Posición del cursor en coordenadas del mundo.
- Conteo de triángulos de la escena.
- FPS medido con `std::time::Instant`.

### F1.4 — Gizmos de Transformación
- Ejes X/Y/Z con handles arrastrables (traslación).
- Aplicado al objeto seleccionado.
- Lógica de picking: ray casting desde el cursor → `kpe-geometry/src/intersection.rs`.

---

## Fase 2 — Pipeline Core → Renderer completo

> **Objetivo:** El usuario puede crear sketches, extrusiones, y booleanos desde la UI. El renderer refleja cambios en tiempo real.

### F2.1 — Document Model en `kpe-app`

```rust
pub struct Document {
    pub feature_tree: FeatureTree,        // árbol de features (kpe-schema)
    pub evaluated:    SceneGeometry,      // meshes evaluados (resultado del core)
    pub selection:    Vec<NodeId>,        // selección actual
    pub history:      CommandHistory,     // undo/redo stack
}

pub struct SceneGeometry {
    pub meshes: HashMap<NodeId, TriangleMesh>,
}
```

Cada vez que un parámetro cambia:
```
1. Document::apply_command(cmd)
2. FeatureTree::evaluate() → SceneGeometry
3. Renderer::upload_mesh() para meshes afectados
4. UI re-render
```

### F2.2 — Command Pattern (Undo/Redo)

```rust
pub trait Command {
    fn execute(&self, doc: &mut Document);
    fn undo(&self, doc: &mut Document);
    fn description(&self) -> &str;
}
```

Comandos iniciales:
- `SetParameterCommand { node_id, param, old_val, new_val }`
- `AddFeatureCommand { feature }`
- `DeleteFeatureCommand { node_id }`

### F2.3 — Evaluación incremental

Cuando cambia un parámetro, solo se re-evalúan los nodos del `FeatureTree` que dependen de ese parámetro (no toda la escena). Usa hash de parámetros para detectar qué nodos están stale.

### F2.4 — Sketch Editor embebido

El editor 2D de sketches se renderiza en el viewport 3D:
- Grid del plano activo (XY/XZ/YZ o plano arbitrario).
- Entidades 2D (líneas, arcos, círculos) renderizadas como líneas finas en wgpu.
- Constraints mostradas como íconos sobre las entidades.
- Activación: doble-click en un feature `Sketch` en el Scene Tree.

---

## Fase 3 — Export y Persistencia

### F3.1 — Guardar / Cargar documento (`.kpe`)

```rust
// KPERecipe serializado a JSON
serde_json::to_writer_pretty(file, &document.feature_tree)?;
```

Diálogo de archivo nativo usando `rfd` (Rust File Dialog — compatible con Windows/Mac/Linux).

### F3.2 — Exportación desde menú

Menú File → Export:
- **STL binario** → `kpe-fabrication`
- **OBJ + MTL** → `kpe-fabrication`
- **DXF 2D** (proyección de cara seleccionada) → `kpe-fabrication`

Diálogo nativo con `rfd`. Escritura directa al archivo sin pasar por JS.

---

## Fase 4 — Performance

> **Objetivo:** escenas con 2M+ triángulos a 60fps constantes.

### F4.1 — Frustum Culling

Antes de renderizar, descartar meshes cuyo bounding box no intersecta el frustum de la cámara. Calculado en CPU, render call solo para los visibles.

### F4.2 — LOD (Level of Detail)

Tessellation adaptativa según distancia de cámara (ya propuesto en el roadmap de visión). El renderer solicita al core el LOD apropiado según zoom.

### F4.3 — GPU Instancing

Para escenas con muchas repeticiones del mismo mesh (pattern/array de agujeros, tornillos iguales, etc.), usar `wgpu` draw calls instanciados con transformaciones en buffer.

### F4.4 — Ambient Occlusion (SSAO)

Post-proceso opcional para mejorar la percepción de profundidad. Implementado como renderpass adicional en wgpu.

---

## Orden de Implementación

```
Semana 1:  F0.1 + F0.2 — crate kpe-app, estructura de proyecto [✔️ HECHO]
Semana 2:  F0.3 + F0.4 — Render 3D vivo, UI Egui y conexión Core-State [✔️ HECHO]
Semana 3:  F1.1 + F1.2 + F1.3 — layout base con paneles egui
Semana 4:  F1.4 — gizmos de transformación
Semana 5:  F2.1 + F2.2 — document model + undo/redo
Semana 6:  F2.3 + F2.4 — evaluación incremental + sketch editor
Semana 7:  F3.1 + F3.2 — save/load + export
Semana 8+: F4.x — performance optimizations
```

---

## Lo que NO va en el desktop app (principio de separación)

El renderer (`kpe-app`) **nunca debe**:
- Contener lógica geométrica.
- Llamar directamente a Manifold.
- Modificar el `FeatureTree` fuera de `Command::execute`.
- Saber qué tipo de objeto está renderizando (solo recibe `TriangleMesh`).

El core (`kpe-geometry`, `kpe-parametric`) **nunca debe**:
- Importar `wgpu`, `egui`, `winit`, ni ningún crate de UI/render.
- Tener estado de cámara o selección (eso es del renderer).

---

## Referencias

- wgpu: <https://wgpu.rs>
- egui: <https://github.com/emilk/egui>
- egui-wgpu: <https://docs.rs/egui-wgpu>
- winit: <https://github.com/rust-windowing/winit>
- rfd (file dialogs): <https://github.com/PolyMeilex/rfd>
- Learn wgpu tutorial: <https://sotrh.github.io/learn-wgpu>
