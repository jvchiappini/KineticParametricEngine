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
  viewport_selection()             → ray-AABB picking on left-click
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
│  TopBottomPanel::top("toolbar")                      │
├──────┬───────────────────────────┬──────────────────┤
│Left  │                           │   Right           │
│Panel │     3D Viewport           │   Panel           │
│scene │     (Bevy Camera)         │   properties      │
│tree  │     + grid + axis gizmos  │                   │
│      │                           │                   │
├──────┴───────────────────────────┴──────────────────┤
│  TopBottomPanel::bottom("status_bar")                │
└─────────────────────────────────────────────────────┘
```

When sketch editor is active, only `status_bar` is shown; sketch has its own `TopBottomPanel::top("sketch_toolbar")` and `SidePanel::right("sketch_constraints")`.

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
viewport_selection (on left-click, not in panel areas):
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

All mutations (including Group/Assembly/Fillet/Chamfer wraps) now go through CommandHistory.

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
