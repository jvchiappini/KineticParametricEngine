# KPE — Project Knowledge Base

## Stack (current, May 2026)

- **Desktop**: Bevy 0.15 + egui 0.30 + Rust (100% native, primary target)
- **Web**: React + Three.js + WASM (deprecated, not maintained)
- **Geometry kernel**: Manifold (`manifold-csg` crate) + `csgrs` fallback
- **Serialization**: JSON via serde, `.kpe` files

## Repository Layout

```
KineticParametricEngine/
├── apps/
│   ├── desktop/          # ← Primary app (Bevy + egui)
│   │   └── src/
│   │       ├── main.rs           # App setup, keyboard shortcuts
│   │       ├── app.rs            # AppState resource, ui_system
│   │       ├── commands.rs       # Command trait, undo/redo, all commands
│   │       ├── document.rs       # Document, SceneGeometry, evaluation
│   │       ├── sync.rs           # Bevy mesh/material sync
│   │       ├── camera.rs         # OrbitCamera
│   │       ├── gizmos.rs         # 3D gizmo (translate)
│   │       ├── io.rs             # File I/O (save/load/export)
│   │       ├── sketch_editor.rs  # 2D sketch mode (Bevy + egui)
│   │       └── ui/
│   │           ├── toolbar.rs    # Top toolbar
│   │           ├── scene_tree.rs # Left panel
│   │           ├── properties.rs # Right panel
│   │           └── status_bar.rs # Bottom bar
│   └── web/              # Deprecated React app
├── crates/
│   ├── kpe-schema/       # Core data types (GeometryNode, Joint, Recipe)
│   ├── kpe-geometry/     # Mesh building, CSG, extrude, sketch solver, JointEngine
│   ├── kpe-parametric/   # Expression eval, RuleEngine, Solver
│   ├── kpe-material/     # Procedural texture generation
│   ├── kpe-fabrication/  # Cutlist, nesting
│   ├── kpe-wasm/         # WASM bindings (for web app)
│   └── kpe-cli/          # CLI tools
└── wiki/                 # Documentation & knowledge
```

## Key Architectural Decisions

### Scene Tree
- Root is a `GeometryNode` with `node_type: Compound`
- All nodes have: `id`, `node_type`, `transform`, `children`, `operations` (CSG), `color`
- Tree evaluated recursively: each non-container node produces a `TriangleMesh`
- Container types (`Compound`, `Assembly`) merge children's meshes

### Transform Pipeline
- Each node has `transform: Option<TransformOp>` (translation, rotation, scale)
- Local matrix computed from TransformOp; accumulated as `world = parent_world * local`
- Joints add `world * joint_matrix` between parent and child
- All transforms baked into vertex positions (no GPU transform hierarchy)

### Evaluation & Caching
- `evaluate_all()` computes world matrices, then evaluates each non-container node
- Hashes detect unchanged nodes → skip rebuild (stale meshes reused)
- `evaluate_node(id)` re-evaluates a single node
- Bevy `sync_meshes` compares `mesh_gen` counter → updates GPU meshes

### Undo/Redo
- `Command` trait: `execute`, `undo`, `description`
- `CommandHistory` with 50-max undo stack
- `SetParameterCommand`, `AddFeatureCommand`, `DeleteFeatureCommand`, etc.
- All commands execute immediately and are pushed to undo stack

### Joint System
- `KPERecipe.joints: Vec<Joint>` at top level
- Each joint: `parent_id`, `child_id`, `type` (Revolute/Prismatic/Fixed/Ball), `pivot`, `axis`, `limits`, `current_value`
- `JointEngine::compute_joint_matrix()` returns `DMat4`
- Joint transforms applied during mesh evaluation (baked into vertices)

### Material/Color
- `color: Option<String>` (hex like `"#FF8800"`) on each `GeometryNode`
- Bevy sync creates per-node `StandardMaterial` via `cache.materials`
- Color picker in properties panel (egui `color_edit_button_srgb`)

## Session History

### Session 1 (May 2026) — Phase 1
- Multi-delete sketch: `delete_selected()` + borrow fix
- Ctrl+A select all: collect IDs first to avoid E0502
- Parametric editing: double-click constraint marker popup (Distance/Angle/Radius)
- Phase 1 roadmap marked complete

### Session 2 — Phase 2 (Duplicate/Array, Mirror, Fillet/Chamfer, Extrude taper, Material)
- Duplicate (Ctrl+D): `duplicate_selected()` with ID reassignment
- Array: dialog with count + translation/rotation/scale offsets
- Mirror: dialog with plane selection → negative scale
- Fillet/Chamfer 3D: `FilletDef`/`ChamferDef` schema → modifier nodes wrapping children
  - Manifold: `smooth_out()` + `refine_to_length()` approximation
  - csgrs: pass-through
- Extrude taper: `taper_angle` → ring subdivision with linear scale
- Material picker: `color: Option<String>` → egui color picker → Bevy per-node material
- All 53 tests pass

### Session 3 — Phase 3 (Assembly System)
- `Assembly` variant on `GeometryNodeType` with `AssemblyDef { name }`
- Mesh builder accepts `joints`, applies `JointEngine` during traversal
- `add_assembly()` wraps node in-place (no ID reassignment)
- `add_group()` wraps node in Compound
- `add_joint()` creates Joint via dialog
- Scene tree shows Joints section, properties shows joint slider
- Delete key + right-click context menu + "Delete" button
- Sketch flicker fixed: main UI panels hidden during sketch mode

### Session 4 — Professional Features & Polish
- **View presets**: keys 1 (Front), 2 (Top), 3 (Right), 4 (Iso) set OrbitCamera target/distance/yaw/pitch
- **3D viewport selection**: left-click ray-AABB intersection against meshes (closest hit → `state.document.selection`)
- **Multiple selection (Ctrl+click)**: `multi_selection: Vec<String>` on Document; Ctrl+click toggles items in scene tree; `delete_selected_nodes()` deletes all selected; transforms affect primary selection
- **Auto-save**: 120-second Timer writes `%APPDATA%/KPE/kpe_autosave.kpe`
- **Constraint list panel**: right-side panel in sketch mode lists all constraints with delete via right-click context menu; click to edit Distance/Angle/Radius values
- **MeshNodeId component**: maps Bevy entities back to scene node IDs for raycasting
- **Fixed Fillet/Chamfer undo**: redo stack is now properly cleared (was bypassing `history.execute`)
- **Fixed add_group/add_assembly undo**: now use `DeleteFeatureCommand` + `AddFeatureCommand` via history instead of direct mutation (Group and Assembly wrapping are now undoable)
- **Fixed Ctrl+A**: selects all nodes into `multi_selection` instead of just setting selection to "Root"
- **Fixed F fit-all**: computes actual content bounding box from mesh AABBs instead of always resetting to origin
- **3D viewport grid**: ground-plane grid drawn via Bevy Gizmos (10x10, 1m spacing, axis lines highlighted)
- **Axis indicator**: XYZ axis indicator at origin with colored spheres (red X, green Y, blue Z)
- **Tooltips**: all buttons in toolbar and scene tree have `.on_hover_text()` descriptions
- **File path tracking**: `Document.file_path: Option<String>` remembers where the file was saved; `Document.is_modified: bool` tracks unsaved changes; window title shows `KPE Desktop - Filename*` pattern
- **Empty scene on startup**: no longer creates default Box + Sketch; starts with empty Root node
- **Visibility toggle**: `[v]` / `[ ]` toggle in scene tree per node; hidden nodes are skipped during mesh sync (not rendered)
- All 53 tests pass; `cargo check` clean

### Session 5 — Export, Snap, Undo, About, Camera buttons, Wiki
- **DXF export**: `export_dxf()` writes 3DFACE entities in DXF R12 format for CNC/laser cutting
- **SVG export**: `export_svg()` renders triangle edges as SVG polygons with auto-fit viewBox
- **Save tracks file path**: save now sets `document.file_path` and resets `is_modified`; open doc restores path
- **Sketch undo via CommandHistory**: "Finish" creates `SetSketchCommand` with old/new sketch def → undoable
- **Grid snap in sketch**: "Snap" checkbox snaps mouse to grid (0.5 default); toggle in sketch toolbar
- **View preset buttons**: Front/Top/Right/Iso buttons in toolbar (mirrors 1-4 keys)
- **About dialog**: "?" button shows KPE version, stack info, export formats
- **Camera grid/axis skip during sketch**: viewport_grid and axis_indicator check `editor.active`
- **ViewPreset via AppState**: `pending_view_preset: Option<u8>` bridges egui toolbar → Bevy camera system
- **9 semantic wiki files created**: DATA_MODEL, COMMAND_PATTERN, MESH_PIPELINE, UI_ARCHITECTURE, SKETCH_SYSTEM, CAMERA, SELECTION, FILE_FORMAT, KNOWN_ISSUES (all English)
- All 53 tests pass; `cargo check` clean

## Known Gaps (for next session)

### Critical
- **No Feature Tree / Construction History**: cannot reorder operations, sketches after extrude don't propagate
- **CSG Kernel**: `csgrs` BSP tree produces Z-fighting/inverted faces; needs `manifold` integration for reliable boolean ops
- **Sketch Solver**: gradient descent diverges on cyclic/overconstrained systems; needs Newton-Raphson
- **No STEP/DXF/SVG export**: required for professional CAD interchange

### Polish
- **No undo for sketch edits**: committing sketch geometry bypasses CommandHistory
- **No grid snap**: visual grid exists in sketch mode but doesn't constrain mouse
- **No spline/bezier curves** in sketch editor
- **No trim/extend/offset** sketch tools
- **No face/edge highlighting** on 3D hover
- **No material preview**: ProceduralMaterial exists in schema but has no UI
- **Scene tree**: no drag-and-drop reorder, no search/filter, no expand/collapse all
- **No confirmation dialogs** (delete, unsaved changes)
- **No recent files** or welcome screen

### Performance
- All evaluation blocks the frame (no background threading)
- No LOD/adaptive tessellation
- No frustum culling
