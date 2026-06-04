# Master Architectural Roadmap: Industrial-Grade Sketch Engine (KPE)

Este documento maestro purga los prototipos iniciales y define la hoja de ruta definitiva para evolucionar el **Sketch Engine 2D** de la fase experimental a un estándar **Industrial AAA** equiparable a NX, Fusion360, SolidWorks y AutoCAD.

La filosofía rectora es el rigor y la precisión matemática: Cero inyecciones de código local en las vistas, delegación topológica al motor Rust `kpe-geometry` y una experiencia de usuario estricta orientada hacia la mecanización.

---

## FASE 1: NÚCLEO MATEMÁTICO Y MANEJO DE RESTRICCIONES (CONSTRAINT SOLVER)

### 1.1 Solucionador Analítico Exacto (Newton-Raphson)
El solver empírico o de gradiente simple será descartado. Se construirá un analizador matricial multivariado capaz de asimilar matrices Jacobianas para la resolución no lineal de restricciones geométricas. 
* **Objetivo:** Lograr convergencia y estabilización geométrica en escala de microsegundos con precisión de doble flotante, garantizando que modelos altamente complejos no entren en espirales de oscilación o fallas de borde.

### 1.2 Diccionario Global de Restricciones Rígidas
El motor Paramétrico asimilará en su estructura un catálogo axiomático inquebrantable de relaciones bidimensionales:
* Tolerancias de posición: *Coincidencia, Punto medio, Colinealidad.*
* Tolerancias de orientación: *Paralelismo, Perpendicularidad.*
* Tolerancias de magnitud: *Longitud igual, Radio igual/Corradios.*
* Tolerancias formales: *Tangencia continua cruzada (Línea-Arco, Arco-Arco), Concentricidad.*

### 1.3 Perfilado de Grados de Libertad y Semiótica Visual (DoF)
El backend emitirá evaluaciones topológicas activas informando a la inferfaz Egui del estado termodinámico de las variables.
* **Geometría Azul:** Grados libres descubiertos; la malla permite ser deformada orgánicamente con el ratón.
* **Geometría Negra:** Restringida Completamente (Fully Constrained); anclada matemáticamente al origen espacial.
* **Geometría Roja:** Conflictos de Condicionamiento (Over-constrained) o paradojas topológicas irreconciliables.

---

## FASE 2: MOTOR TOPOLÓGICO DE REGIONES Y DCEL

### 2.1 Lista de Doble Arista y Grafo Planar (DCEL)
El boceto dejará de existir como formas vectoriales vacías flotantes. Será gobernado por un estructurador estricto de topología. Todo arco y toda recta proyectados formarán colisiones algebraicas creando nodos fraccionados. Incesantemente las curvas cerrarán contornos perimetrales (Faces) permitiendo que cualquier conjunto intrincado sea particionado en celdillas atómicas detectables instantáneamente por un algorítmo caminador (*Graph Walker*).

### 2.2 Inferencia de Paridad Automática de Agujeros (Holes vs Islands)
Cuando se trace geometría anidada (ej. un círculo contenido nativamente inter-estructurado por un rectángulo), el compilador resolverá un test computacional tipo *Ray-Cast / Point-in-Polygon* dictaminando el árbol jerárquico. Las capas de profundidad alternarán implícitamente entre Región Sólida y Región Vacía, orquestando mallas paramétricas perfectamente perforadas.

### 2.3 Autoridad de Relleno Transaccional del Usuario (Face Overrides)
Conmutadores transaccionales locales por cada sub-polígono que se genere. El usuario seleccionará las islas orgánicas y aplicará override para que dejen de acatar las reglas Even-Odd paritarias y sean dictaminadas de forma forzada como entidades Sólidas, Huecas, o Transparentes a la hora del empuje dimensional 3D.

---

## FASE 3: DRAFTING PERFECTO DE NIVEL INDUSTRIAL

### 3.1 Motor de Captura Exacta de Instancias (OSnap / Object Snap)
El subsistema interceptará el plano focal trazando proyecciones desde el cursor hacia las ecuaciones matemáticas adyacentes para asegurar fijación nanométrica en:
* *Extremos, Puntos Medios y Centros Analíticos.*
* *Intersecciones Visuales y Aparentes.*
* *Puntos Cuadrantes Circulares y Curvaturas Tangenciales Retardadas.*

### 3.2 Display Dinámico Integral (Heads-Up Display - HUD)
Supresión del "dibujo arrastrando". Junto a la mirilla principal se acoplarán flotadores (Widgets) interactivos nativos exigiendo las variables numéricas dimensionales. Al tipear cantidades como "50 mm" y presionar *Tab*, la magnitud actual queda trancada (locked), trasladando la energía condicional pura a enfocar la dirección o los grados polares del vector.

### 3.3 Rastreo Magnético Ortogonal y Polar
Mientras se orquesta la operación de despliegue dimensional, las guías proyectuales virtuales aparecerán en el lienzo cuando el cursor entre dentro del delta Epsilon permisible de ángulos críticos pre-programables por la manufactura espacial (0°, 30°, 45°, 90°, etc.), permitiendo fijación directa al plano constructivo.

---

## FASE 4: EXPULSIÓN DE CÓDIGO DIRECTO HACIA LA EXTRUSIÓN ACTIVA

### 4.1 Deconstrucción Plástica / Push-Pull Modelado
El flujo de usuario adoptará el comportamiento orgánico *SketchUp UX*. La varita detectora resaltará instantáneamente las topologías de las Caras (Faces DCEL). Una vez capturadas, el comando 3D no extruirá un "Dibujo General", sino que instanciará objetos tridimensionales basados en la semilla vectorial extraída.

### 4.2 Proyección por Revestimientos Complejos de Extrusion, y Revolución (Sweeps & Revolves)
Sincronización natural del árbol perimetral de bocetos 2D con los algoritmos booleanos del motor B-Rep/CSG profundo. Los ejes de construcción dibujados en el sketch 2D mutarán de forma instantánea a Pivotes Vectoriales sobre las caras anidadas detectadas, facilitando revoluciones industriales (Revolves) sobre centros abstractos con simetrías matemáticas aseguradas.

---

## FASE 5: ARQUITECTURA DE RENDIMIENTO RADICAL E INMUTABILIDAD

### 5.1 Jerarquía Estructural BVH (Bounding Volume Hierarchy)
La selección ciega O(n) sobre todos los subelementos matemáticos pasará a ser O(log n). El motor 2D empaquetará todas las cuerdas referenciales en cajones de encapsulación binarios espacialmente referenciados indexando instantáneamente miles de geometrías concurrentes, asegurando inercia inamovible frente al consumo intensivo de CPU.

### 5.2 Purga de la Interfaz UI a Capa Estéril (Domain Segregation)
Todo el sistema listado reside y residirá pura y exclusivamente encapsulado en los procesadores del *Crate* principal `kpe-geometry`. Egui, y cualquier tecnología de Frontend anexa, estarán rebajadas e instruidas mediante directivas explícitas únicamente a pintar por transferencia de estados inmutables y capturar comandos; sellando así la arquitectura AAA donde ningún aspecto topológico, relacional, matricial o lógico pueda sufrir fugas entre dominios computacionales.
