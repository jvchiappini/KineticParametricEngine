# Roadmap del Motor Paramétrico y Sketch Engine (KPE)

Este documento detalla los lineamientos y las profundas mejoras arquitectónicas necesarias para que el sistema de bocetos 2D (Sketch Engine) y su resolutor geométrico abandonen la etapa de prototipado y dominen el estándar industrial *AAA* utilizado en software CAD profesional masivo (NX, SolidWorks). No se incluye código, sino directivas del sistema a nivel arquitectónico.

---

## 1. Solucionador Exacto vía Newton-Raphson (Eliminación del Descenso de Gradiente)
*   **Archivos a Modificar:** 
    *   `crates/kpe-geometry/src/sketch/solver.rs`
*   **Acción Requerida:** El motor numérico iterativo basado en perturbaciones (derivadas por diferencia finita) y avance por gradiente debe eliminarse en favor de un sistema de ecuaciones no lineales. Se debe codificar el desarrollo analítico del modelo matemático y utilizar el método multivariado recursivo algorítmico **Newton-Raphson** (o Levenberg-Marquardt), pre-computando la Matriz Jacobiana Analítica.
*   **Beneficio Mantenibilidad / Técnico:** Se extirpa masivamente el estrangulamiento de divergencia por "stiffness" matemático, lo que significa que el CAD jamás se oscilará de manera eterna por desequilibrio en las escalas y longitudes. Convergencia sub-milisegundo precisa en O(1) relativo al salto, posibilitando bosquejos hiper-condensados.

## 2. Retroalimentación Topológica mediante Grados de Libertad (DoF) Visuales
*   **Archivos a Modificar:** 
    *   `apps/desktop/src/sketch_editor/ui.rs` (Para el rendering diferencial a nivel usuario)
    *   `apps/desktop/src/sketch_editor/state.rs`
    *   `crates/kpe-geometry/src/sketch/solver.rs` (Para exportar la clasificación de nodos y estado matricial)
*   **Acción Requerida:** El backend matemático debe dictaminar si los enlaces de una figura alcanzan la inmovilización paramétrica para reflejar de forma óptica cómo las matemáticas visualizan el mundo interno.
    *   Contorno **Azul/Claro**: Grados libres a movimiento.
    *   Contorno **Negro/Oscuro**: Geometría anclada puramente (Fully Constrained).
    *   Contorno **Rojo**: Bloqueo o bucle absurdo paramétrico (Ej. Dos círculos forzados concéntricos sin pertenecer).
*   **Beneficio Mantenibilidad / Técnico:** Saca del oscurantismo visual al usuario. La app previene en caliente que los diseñadores cometan crímenes de proyección sin enterarse, erradicando un factor inmenso de bugs y falsas quejas sobre el software.

## 3. Barras de Tareas para Inyección de Restricciones Explícitas Acopladas
*   **Archivos a Modificar:** 
    *   `apps/desktop/src/sketch_editor/ui.rs`
    *   `apps/desktop/src/sketch_editor/input.rs`
*   **Acción Requerida:** Instalar submenús flotantes Egui reactivos o botones fijos permanentes en la barra superior capaces de inyectar imposiciones geométricas no orgánicas. Tras activar en el modelo `Select` (input ciego) variables o entidades discordantes, el UI exigirá opciones algorítmicas combinatorias exclusivas (Tangencial, Concordancia, Fijación Fija X/Y, Concetricidad).
*   **Beneficio Mantenibilidad / Técnico:** Transforma la consola de dibujo digital desde un "Canvas pintable autoadaptativo" al status absoluto de Bocetador Mecánico Mecanizado.

## 4. Estructuras de Aceleración Espacial frente a Búsqueda Geométrica Ciega
*   **Archivos a Modificar:** 
    *   `apps/desktop/src/sketch_editor/input.rs` (Gestor raycast de clicks)
    *   `crates/kpe-geometry/src/sketch/entities.rs` (Lugar para montar el espacio dimensional indexado)
*   **Acción Requerida:** Erradicar radicalmente la fuerza bruta lineal O(n) (es decir, re-iterar arrays de puntos, arcos, y curvas indiscriminadamente ante eventos rápidos o leves paneos de puntero de Mouse). Reemplazar con modelado topológico anidado tal como un Árbol Binario de Envolvencia **BVH (Bounding Volume Hierarchy)** o indexación **QuadTree**.
*   **Beneficio Mantenibilidad / Técnico:** Resistencia masiva algorítmica al ahogamiento de CPU. La velocidad de percepción de selección no decaerá sin importar si el fichero industrial escala a cargar tableros que integren más de 15.000 curvas interpoladas en simultáneo en el mismo Canvas Paramétrico.
