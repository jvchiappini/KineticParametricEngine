# Master UI/UX Architectural Roadmap: Industrial-Grade KPE Desktop

Este documento maestro define la visión definitiva y los lineamientos de diseño de la interfaz y experiencia de usuario (UI/UX) para la aplicación de escritorio de KPE. El objetivo irrenunciable es abandonar el aspecto de "herramienta de desarrollador" y ascender al pináculo del estándar **Industrial AAA**.

La interfaz debe ser estrictamente funcional, de máxima densidad de información, responsiva, y orientada a **Power-Users** de diseño mecánico: fluida para principiantes, pero devastadoramente rápida y orientada a atajos para profesionales (como Blender, Fusion 360 o NX).

---

## FASE 1: NAVEGACIÓN Y GEOMETRÍA VISUAL (VIEWPORT UX)

### 1.1 Sistema de Renderizado de Selección de Precisión
Abandonaremos el highlighting rudimentario que repinta toda la malla de un color plano.
* **Acción:** Implementar un renderizado multipaso para el resaltado de selección. El componente seleccionado debe mostrar un contorno delimitador (Outline Stroke) limpio y preciso (anti-aliased) que respete la profundidad (Depth-Aware), permitiendo entender qué partes están ocluidas detrás de otras geometrías sin obstruir la lectura del volumen 3D absoluto.

### 1.2 Viewport Gizmos e Interacción Táctil Matemática
El motor de manipulación tridimensional.
* **Acción:** Instalar Gizmos paramétricos magnéticos de seis grados de libertad. Al cliquear las flechas o anillos, la interacción debe bloquear el movimiento al plano o eje exacto. El Gizmo presentará un *Snap-Grid* invisible (configurable por el usuario) durante su uso, mostrando una regla flotante traslúcida (*Ruler Override*) que trace la magnitud espacial desplazada en tiempo real (milisegundos).

### 1.3 ViewCube y Menú HUD Ortogonal Intrusivo Absoluto
* **Acción:** Reemplazar los botones primitivos (Top, Front, Iso) por un *NavCube* dinámico en la esquina de la pantalla. Al hacer clic en vértices, aristas o caras del cubo, la cámara transicionará con interpolación matemática suave entre ángulos ortogonales y perspectivas isométricas absolutas.

---

## FASE 2: EL ÁRBOL DE ESCENA SEMÁNTICO Y EL GESTOR DE OPERADORES

### 2.1 Drag and Drop Interactivo Global (Árbol de Comandos)
La jerarquía de la escena debe evolucionar hacia un sistema de ensamblaje masivo.
* **Acción:** El *Scene Tree* soportará selección cruzada masiva, desplazamiento visual de nodos arriba o abajo insertándolos *dentro* o *fuera* de componentes. Cada arrastre en la UI se inyectará como un comando puro determinista y reversible hacia el `kpe-parametric`.

### 2.2 Micro-Switches Locales de Estado (Visibility & Compute)
* **Acción:** Cada nodo del árbol poseerá indicadores minimalistas rápidos para operaciones frecuentes: 
    * "Ojo" (Visibilidad global)
    * "Candado" (Inmutabilidad paramétrica protegiendo de clics accidentales)
    * "Trueno" (Suppress/Unsuppress de evaluación en el árbol de operaciones paramétricas).

---

## FASE 3: HEADS-UP DISPLAY (HUD) IN-CONTEXT Y COMMAND PALETTE

### 3.1 Menús Contextuales Flotantes Inteligentes (HUD)
El usuario jamás deberá trasladar la vista desde el centro del modelado (Viewport 3D) hacia paneles en los extremos del monitor para operaciones elementales.
* **Acción:** Al seleccionar caras o nodos, invocar un HUD radial o flotante transparente adyacente al cursor (tipo *Marking Menu* de Maya o HUD radial de Fusion), ofreciendo de forma instintiva operaciones ligadas taxonómicamente a la selección actual (ej: Extrude inmediato si se seleccionó una cara 2D, Fillet inmediato si se seleccionó un nodo 3D Edge).

### 3.2 Paleta de Comandos Unificada Omnipresente (Slash Commands)
* **Acción:** Acoplar una paleta universal central superpuesta e invocable vía teclado (Ej. presionar <kbd>Espacio</kbd> o <kbd>/</kbd>). Esta consola difuminará en el fondo la aplicación e indexará instantáneamente todo comando del motor (Ej: tipear "chamfer" y enter para aplicar un chaflán masivo) similar al *Command Palette* moderno popularizado por editores AAA, garantizando fluidez sin usar el ratón a más del 90%.

---

## FASE 4: PANEL DE PROPIEDADES REACTIVO Y EDITOR PARÁMETRICO

### 4.1 Entradas de Texto Evolutivas (Math-Bindings)
* **Acción:** Ninguna caja de ingreso métrico (`f64`) estará confinada a ser un espacio rígido tipográfico. Todo "Text Input" en el panel de propiedades se debe comportar al mismo tiempo como:
    * Arrastrador virtual continuo (Slider) tirando y empujando del ratón horizontalmente encima del rótulo general.
    * Intérprete Matemático embebido: permitiendo teclear en tiempo de ejecución "50 * 2 / pi" interpretando transaccional el string a valor final sin software tercero.

### 4.2 Árbol de Estado Interno Dinámico y Desplegable
* **Acción:** Evitar las listas kilométricas verticales clásicas donde la propiedad queda oculta si es muy compleja. Las secciones topológicas con propiedades n-dimensionales (Ej. Los arrays estáticos de Múltiples Pivotes y Coordenadas Locales/World) figurarán como pestañas colapsables asincrónicas agrupadas jerárquicamente, con botones de inyección `[+]` limpios anexados en línea, sin que requieran ventanas emergentes modales (No-Modals Layout).

---

## FASE 5: CUSTOMIZACIÓN DE ESPACIO DE TRABAJO (WORKSPACE DOCKING Y THEMES)

### 5.1 Paneles Encajables y Pestañas Flotantes Multi-Monitor (Docking/Tabbing)
Un AAA es altamente configurable para workflows dual-monitor.
* **Acción:** Sustituir los layouts fijos de columnas estáticas por un sistema modular de Docking de paneles superpuestos, posibilitando desmontar el *Árbol de Escena* o el *Panel de Propiedades* del cliente principal, colapsarlos como solapas, y arrastrarlos autónomamente en ventanas satélite de los sistemas operativos (soporte en multi-monitor de primer nivel).

### 5.2 Estética Dark-Tone Industrial Mínimamente Intrusiva (UI Theme)
* **Acción:** Acordonar Egui a un esquema de color personalizado pre-estipulado (Tema Oscuro por defecto: grises metálicos y acentos orgánicos sutiles). Ningún color fosforescente primitivo sin depurar manchará la estética general. Fuentes de letra tipográficas *sans-serif* racionales con densidades balanceadas, márgenes interlineados matemáticos, y un uso de Iconografía Limpia sin rótulos masivos redundantes. Requerirá diseñar librerías tipográficas propias para KPE en la fuente general vectorial (TTF/OTF escalable por vector).
