# Plan de Ejecución: Migración Cero Fricción (Frontend -> Crates)

*(Este documento ha sido actualizado reflejando los avances recientes del proyecto. La aceleración espacial como el BVH ya ha sido delegada exitosamente al core geométrico, lo que indica que el proceso de limpieza y separación ha comenzado correctamente).*

Este plan táctico aborda de forma secuencial la extracción de la lógica dura que aún sigue atrapada en la aplicación `kpe-app` (Desktop) para que KPE goce de una arquitectura de motor puro.

---

## FASE 1: Trasplante del Evaluador del Grafo Acoplado
Actualmente en el Desktop se realiza el cálculo de matrices combinadas y la resolución de las juntas (Joints). Esto previene que una web app o una consola puedan compilar un `.kpe` sin reescribir esta lógica.

1. **Destino de Traslado:** `crates/kpe-geometry/src/evaluator.rs` (Nuevo módulo a crear).
2. **Archivos a Purgar (Origen):** `apps/desktop/src/document.rs`.
3. **Paso a Paso:**
    *   Mudar las funciones estructurales: `compute_world_matrices`, `hash_geometry_node` y recursiones como `collect_evaluated_meshes`.
    *   Refactorizar `build_mesh_with_joint_context` iterador matemático del AST para que dependa únicamente del esquema central y exija firmas abstractas `(&GeometryNode, &HashMap, &[Joint])`, sin referencia alguna al UI Bevy.
    *   **Punto de Cierre:** El crate `kpe-geometry` exportará una función pública maestra, ej. `kpe_geometry::evaluate_scene(recipe: &KPERecipe) -> SceneGeometry`. La app Desktop se limitará exclusivamente a mapear ese retorno hacia mallas de OpenGL.

## FASE 2: Extracción del Sistema Transaccional y Comandos Paramétricos
Toda transformación, creación y borrado topológico ("Mirror", "Fillet", "Array") es parte integral de un Modelador Paramétrico (CAD) y no pertenece a Botones o Callbacks visuales.

1. **Destino de Traslado:** `crates/kpe-parametric/src/commands/` y `/history/` (Nuevos módulos).
2. **Archivos a Purgar (Origen):** `apps/desktop/src/commands.rs` y `apps/desktop/src/feature_commands.rs`.
3. **Paso a Paso:**
    *   Mudar la máquina de historiales (`CommandHistory`, stacks de Undo/Redo) encapsulándola nativamente en memoria manejable.
    *   Extraer los constructores paramétricos complejos (`add_fillet`, `mirror_selected`, `array_selected`). Eliminar de estos toda mención a `&mut crate::app::AppState`. Estas funciones mutarán directamente a un abstracto `&mut GeometryScene`.
    *   **Punto de Cierre:** Al hacer clic en un botón de la GUI, en vez de arrancar un cálculo morfológico ciego, la interfaz disparará directriz hacia el control topológico: `engine_history.execute(Command::AddFillet(0.5))`.

## FASE 3: Enraizar Matemáticas de Proyección Vectorial (Core Geométrico)
La interfaz necesita utilidades matemáticas para poder saber por qué rayo 3D pasa el mouse, pero esas matemáticas son dependientes del Plano, no del motor de Video.

1. **Destino de Traslado:** `crates/kpe-geometry/src/sketch/plane.rs`
2. **Archivos a Purgar (Origen):** `apps/desktop/src/sketch_editor/math.rs`.
3. **Paso a Paso:**
    *   Mudar definitivamente `to_3d()`, `to_2d()`, `sketch_plane_normal()` y `circle_basis()`.
    *   Implementarlas como Trait base o Patrón Método de `SketchPlane::to_3d(&self, x, y) -> Vec3`. Así cualquier frontend en TypeScript o Rust puede llamar nativamente a la resolución del plano virtual.

## FASE 4: Test de Pureza (Validation Checks)
Al momento de re-enlazar, realiza las siguientes pruebas de humo para asegurar el éxito:

1. **Validación Inversa:** Un simple barrido con *Clippy* que valide que ninguna crate (`kpe-geometry`, `kpe-parametric`) intente invocar nada que provenga de `apps/*` (No Backtracking Dependency).
2. **Prueba Ciega ("Headless Engine Test"):** Escribir una prueba unitaria transaccional (en la crate paramétrica) que ejecute un historial de comandos falso: añadir caja, añadir espejo, invocar evaluador de mallas. Si los vértices se devuelven matemáticamente exactos sin alzar una ventana visual, la migración es un triunfo industrial absoluto.
