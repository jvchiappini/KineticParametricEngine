# Desktop App Architecture

## System Graph (Bevy Update order)

```
Startup:
  setup()                          → camera, lights
  sync::setup_scene()              → SceneMeshRoot entity

Update (each frame):
  app::ui_system()                 → egui panels (toolbar, scene tree, properties, status)
  sync::sync_meshes()              → Bevy mesh/material sync (skips hidden nodes)
  camera::orbit_camera_system()    → mouse orbit/zoom, view presets (1-4), fit-all (F)
  keyboard_shortcuts()             → Ctrl+C/X/V/Z/Y/D/A/S, Delete
  view_preset_handler()            → deferred view preset from UI button
  build_tool::tool_shortcut_system → Space/R/C/L/P/M/E for tool switching; ESC to cancel/cancel
  viewport_selection()             → ray-AABB picking (ONLY when BuildTool::Select active)
  face_pick_system()               → ray-triangle face selection (ONLY when PushPull tool or shift+click)
  push_pull_drag_system()          → drag-to-extrude when face selected
  draw_selected_face_system()      → turquoise gizmo on selected face
  rect_tool_system()               → click-click rectangle creation (ONLY when Rect active)
  circle_tool_system()             → click-click circle creation (ONLY when Circle active)
  tool_render_system()             → gizmo previews for active tool (rubber-band rect/circle)
  tool_palette_ui_system()         → egui floating tool palette window
  auto_save_system()               → 120s timer → %APPDATA%/KPE/kpe_autosave.kpe
  update_window_title()            → "KPE Desktop - Filename*"
  viewport_grid()                  → ground-plane grid via gizmos
  axis_indicator()                 → XYZ axis at origin via gizmos
  gizmos::gizmo_interaction_system()
  gizmos::gizmo_render_system()    → 3D translate gizmo
  sketch_editor::check_enter_sketch_mode()
  sketch_editor::sketch_input()    → Bevy mouse/keyboard for sketch
  sketch_editor::render_sketch()   → Bevy gizmos for sketch
  sketch_editor::render_sketch_wireframes()
  sketch_editor::sketch_ui()       → egui toolbar + constraint panel for sketch
```

## Key Resources

| Resource | Type | Description |
|----------|------|-------------|
| `AppState` | `ResMut` | Document, history, dialogs, mesh generation counter |
| `BuildToolState` | `ResMut` | Active build tool (Select/Rect/Circle/Line/PushPull/Move/Eraser), tool phase, inference text |
| `PushPullState` | `ResMut` | Face selection state, drag origin, accumulated extrusion distance |
| `MeshCache` | `ResMut` | Bevy Mesh handles, entities, materials per node |
| `SketchEditorState` | `ResMut` | Active sketch document, tool, selection |
| `GizmoState` | `ResMut` | 3D gizmo mode/selection |
| `AutoSaveTimer` | `ResMut` | 120-second repeating timer |
| `OrbitCamera` | Component | Camera orbit/zoom state |
| `SceneMeshRoot` | Component | Root entity for all meshed children |
| `MeshNodeId` | Component | Maps entity → scene node ID for ray picking |

## egui Panel Layout

```
──────────────────────────────────────────────────────┐
│  TopBottomPanel::top("toolbar")  (main menu bar)     │
├────────┬───────────────────────────┬─────────────────┤
│Tool    │                           │                 │
│Palette │     3D Viewport           │   Properties    │
│floating│     (Bevy Camera)         │   Panel         │
│window  │     + grid + axis gizmos  │   (right side)  │
│(top-   │     + tool preview gizmos │                 │
│left)   │                           │                 │
├────────┴───────────────────────────┴─────────────────┤
│  TopBottomPanel::bottom("status_bar")                 │
└──────────────────────────────────────────────────────┘
```

When sketch editor is active, only `status_bar` is shown; sketch has its own panels.
When build tools are active, a floating tool palette (minimal, 7 buttons) appears at top-left (220px offset, 42px from top).

## BuildTool System

The `build_tool` module implements SketchUp-style direct 3D manipulation:

### Tools and Keyboard Shortcuts

| Tool | Key | Behavior |
|------|-----|----------|
| Select | Space | Existing AABB ray-picking for node selection |
| Rectangle | R | Click 1 → infer construction plane → Click 2 → create `BoxDef(width, depth, 0.01)` |
| Circle | C | Click 1 → center + plane → Click 2 → create `CylinderDef(radius, 0.01)` |
| Line | L | Click → click → ... → create polyline segments (scaffolded, not implemented) |
| PushPull | P | Click face → drag → extrude mesh or adjust parametric height |
| Move | M | Click node → drag → translate (scaffolded, not implemented) |
| Eraser | E | Click entity → delete node (scaffolded, not implemented) |
| ESC | — | Cancel current tool phase, or return to Select |

### Construction Plane Inference

1. On first click, `infer_plane()` ray-picks the closest face via `pick_face()` (Möller–Trumbore against all scene meshes)
2. The hit face normal becomes the construction plane normal
3. Fallback: ground plane (Y=0) at the ray intersection point
4. `ConstructionPlane { origin, normal, u_axis, v_axis }` provides `project()`, `to_2d()`, `intersect_ray()`

### Tool Gating

Each tool system checks `BuildToolState.active_tool` before responding to clicks:

- `viewport_selection` → only when `BuildTool::Select`
- `face_pick_system` → when `BuildTool::PushPull` OR shift+click
- `rect_tool_system` → when `BuildTool::Rectangle`
- `circle_tool_system` → when `BuildTool::Circle`

## Face Picking + Push-Pull System

The `push_pull_system` module provides face-level ray-triangle picking and mesh extrusion:

```
face_pick_system:
  1. Gate: PushPull tool active OR shift+click
  2. Ray-triangle test (Möller–Trumbore) against ALL scene meshes
  3. Closest hit → record (node_id, face_index, world_hit, face_normal)
  4. Store original mesh for clean re-extrusion

push_pull_drag_system:
  1. Project mouse ray onto plane (drag_origin, face_normal)
  2. Signed distance along normal → extrusion distance (snapped to 1mm)
  3. Re-extrude from original mesh each frame (no error accumulation)
  4. On release: finalize mesh, trigger sync

draw_selected_face_system:
  1. Gizmo wireframe (turquoise) on selected face
  2. Double-line glow effect
  3. Hidden during active drag
```

## Mesh workflow

```
AppState::mark_dirty()
  → mesh_gen += 1, document.is_modified = true
  → sync_meshes detects change
  → iterates state.document.evaluated.meshes
  → skips nodes in document.hidden_nodes
  → creates/updates/removes Bevy Mesh3d entities
  → per-node StandardMaterial from node.color
```

## Viewport selection (ray-AABB)

```
viewport_selection (on left-click, NOT in panel areas, ONLY when Select tool active):
  1. Cast ray from camera through cursor
  2. For each mesh entity with MeshNodeId + Aabb + GlobalTransform:
     a. Transform ray to model local space via inverse matrix
     b. Compute ray-AABB intersection (slabs method)
     c. Pick closest hit
  3. If Ctrl held → toggle in multi_selection
     Else → set primary selection, clear multi_selection
```

## Command pattern

```
Command trait:
  execute(doc)  → modifies scene tree
  undo(doc)     → reverts modification

CommandHistory:
  execute(Box<dyn Command>) → cmd.execute(), push to undo_stack, clear redo_stack
  undo() → pop undo_stack, cmd.undo(), push to redo_stack
  redo() → pop redo_stack, cmd.execute(), push to undo_stack
```

All tool-created nodes (Rectangle → BoxDef, Circle → CylinderDef) go through `AddFeatureCommand` via `AppState::execute()`, making them fully undoable.

## Scene tree evaluation

```
evaluate_all():
  1. compute_world_matrices() → HashMap<node_id, DMat4>
  2. collect_evaluated_meshes() → for each non-container node:
     a. If node is child_id in a joint → apply parent_world * joint_matrix
     b. Else → normal eval (world = parent_world * local)
  3. Store in SceneGeometry { meshes }
  4. Update node_hashes for change detection
```

## Document metadata

- `file_path: Option<String>` — remembers last save/load location
- `is_modified: bool` — set to true by `mark_dirty()`, displayed as `*` in title bar
- `hidden_nodes: HashSet<String>` — node IDs to skip during mesh sync
- Window title format: `"KPE Desktop - {filename}*"`
