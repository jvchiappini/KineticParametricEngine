# Arquitectura UX/UI: El Panel de Sketch (Cinta de Dibujo Dinámica)

Este documento proyecta de forma implacable el estándar absoluto de interfaz gráfica especializada para el entorno **Sketch 2D** dentro de KPE. El propósito es emular y superar el flujo de trabajo sagrado de **AutoCAD** y aplicaciones mecánicas para lograr una experiencia de usuario (UX) extremadamente perfecta, en la que trazar miles de líneas sea un esfuerzo trivial, orgánico y velozmente mecanizado.

Ninguna línea de código habita aquí, sino directrices estrictas de diseño de dominio visual y conductual.

---

## FASE 1: BARRA DE ESTADO DE DIBUJO E INFERENCIA (STATUS BAR / SNAPS)

### 1.1 Conmutadores Magnéticos Visibles (OSnap Toggles)
Inoculado en un panel estático inferior o anclado lateral, el usuario debe tener acceso ocular e interactivo directo y constante a los filtros de inferencia matemática.
* **Acción:** Integrar una tira persistente de íconos que representan palancas on/off (tipo interruptor): *Punto Medio, Extremo, Tangente, Centro, Intersección, Cuadrante*.  
* **Flujo Perfecto:** El dibujante no debe entrar a "Opciones -> Preferencias" para apagar la "Tangencia" que le ensucia la malla. Tan sólo hará un clic en la barra perimetral para apagar temporalmente esa interferencia espacial instantáneamente.

### 1.2 Modalidad Ortogonal y Rejilla Interactiva (Ortho & Grid Snap)
* **Acción:** Acceso universal (mediante tecla directa de funcion `F8` y botón persistente en panel) al cierre de trazos cartesianos absolutos (0°, 90°, 180°, 270°). Junto a él, un ajustador dinámico de rejilla sub-fraccional (Grid Spacing override), forzando escalonamientos milimétricos en el lienzo al hacer zoom inverso y directo sin ahogar la pantalla.

---

## FASE 2: GESTIÓN DE HERRAMIENTAS Y BARRA DE CREACIÓN (RIBBON & TOOLBARS)

### 2.1 Cinta Superior de Herramientas Acopladas (Contextual Ribbon)
Cuando el usuario entra en modo "Boceto Activo" (Edit Sketch), todo el cascarón ajeno del programa debe atenuarse, inyectando un menú horizontal masivo y organizado visualmente en la zona superior.
* **Contenido de la Cinta:** 
  1. *Creación Rápida:* Líneas (Continuas, Splines), Círculos (Centro, 2P, 3P), Arcos.
  2. *Modificación Activa:* Offset (Desfase en cadena), Trim (Recorte de poder arrasando geometría cruzada), Enpalme (Fillet).
  3. *Restricciones Duras:* Bloques de candados topológicos (Fijar Paralelo, Tangente mutua, Concéntrico).

### 2.2 Panel de Opciones de Herramienta Transitoria (Tool Property Inspector)
El AutoCAD brilla porque las herramientas tienen su propio submundo.
* **Acción:** Si se clickea el botón "Círculo", el Panel de Propiedades abandona su estado global y entra exclusivamente en "Opciones de Círculo". Esto permite que el usuario decida parámetros estipulados (Ej: Forzar a dibujar el círculo mediante Diámetro o Radio, activar si es una línea constructiva (punteada) y no de extrusión pura) ANTES de tocar el canvas.

---

## FASE 3: MÁQUINA DE ESTADOS Y CONDUCTA DE COMANDOS (THE AUTOCAD WORKFLOW)

### 3.1 Encadenamiento Infinito de Herramientas (Continuous Command Loop)
Nada mata la fluidez más rápido que reactivar la herramienta.
* **Acción UX Perfecta:** Trazar la herramienta "Línea" no diseña una línea y termina. Lanza una Máquina de Estados que repite un rastreo tras arrancar su nuevo nodo. Al clickear otra vez, traza esa recta y su tramo final sirve en caliente como punto de largada del siguiente eslabón. Se mantiene viva indefinidamente hasta que el usuario presione irrevocablemente <kbd>ESC</kbd> o asiente con Intro/Clic-Derecho.

### 3.2 Indicadores Flotantes Dinámicos de Comando (Cursor Tooltips)
* **Acción:** En todo momento existirá una viñeta flotante milimétrica pegada por un vértice a la punta del puntero del mouse que instruirá textualmente al operario sobre qué coordenada espera la máquina lógica.
   * *Ejemplo paso 1:* "Especifique primer punto (X,Y) o Seleccione entidad..."
   * *Ejemplo paso 2:* "Especifique longitud y delta (L ∠ A)..."
Este mini panel flotante no solo dictará el paso actual del comando invocado, sino que hospedará los recuadros resaltados mencionados en tu Head-Up Display global.

---

## FASE 4: SELECCIÓN ESPACIAL AVANZADA INTER-SKETCH (MARQUEE SELECTION)

### 4.1 Ventana Cruzada versus Ventana Estricta (Crossing vs Window Select)
La base del diseño 2D en AutoCAD industrial se sustenta en cómo cruzas el ratón para hacer selecciones múltiples.  
* **Acción UI:** Se incorporará un algoritmo matemático de arrastre sobre el canvas plano.
  * **De Izquierda a Derecha (Azul):** `Window Selection`. Sólo atrapará perfiles y restricciones que queden sumergidas y envueltas al 100% dentro del rectángulo contenedor.  
  * **De Derecha a Izquierda (Verde):** `Crossing Selection`. Abrazará masivamente perfiles, puntos o arcos con el sólo acto de tropezar y morder con el alambre visual del límite cualquier trozo diminuto del elemento matemático dibujado.

---

## FASE 5: REVESTIMIENTO COSMÉTICO Y JERARQUÍA ESQUEMÁTICA (CODE-CLEAN)

### 5.1 Feedback Visual en Fríos Matemáticos (Color Grading UX)
Los diseñadores actúan con inercia e iteración. 
El panel y el Lienzo (Canvas del Sketch) usarán pesos de grosores escalados dinámicamente independientes del Zoom, garantizando que el alambre base no mute de grosor al ahondar en el componente, mientras colorea por taxonomías.
1. Gris tenue translúcido: Líneas Constructivas (Construction Geometry / Ejes guías).
2. Turquesa reactivo fluorescente: Selección actual y Highlight bajo impacto de cursor (Hover states).
3. Púrpura sutil o rellenado tramado cruzado de baja opacidad: Reconocimiento inteligente de Caras Aisladas interceptadas (El DCEL funcionando interactivamente, avisándole al operario "Esta será un área extruíble").
