# UI Architecture

## egui Panel Layout

The UI is built entirely with egui panels, composed in `ui_system()` (`app.rs:46`). The main layout has four panels:

### Toolbar (top)

`ui/toolbar.rs` — `TopBottomPanel::top("toolbar")` — Contains:
- Document-level buttons: New, Open, Save
- Undo/Redo buttons bound to `CommandHistory`
- Export menu (STL, OBJ)
- Triangle count readout

### Scene Tree (left)

`ui/scene_tree.rs` — `SidePanel::left("scene_tree")` — Contains:
- Recursive `tree_node()` display of the entire `GeometryNode` tree
- Joint list with selection
- Action buttons: Copy, Cut, Paste, Duplicate, Delete
- Feature buttons: Array, Mirror, Fillet, Chamfer dialogs
- Primitive creation buttons: +Box, +Cyl, +Sph, +Sketch
- Group and Assembly wrapping buttons
- Modal dialogs for Array, Mirror, Fillet, Chamfer, and Joint (rendered as `egui::Window`)

### Properties (right)

`ui/properties.rs` — `SidePanel::right("properties_panel")` — Per-type editors:
- Box: Width, Height, Depth drag-values
- Cylinder: Radius, Height
- Sphere: Radius
- Extrude: Distance, Taper angle
- Revolve: Angle
- Fillet/Chamfer: readonly display of radius/distance
- Transform: Position, Rotation (degrees), Scale axis editors
- Color picker (hex → srgb picker → hex roundtrip)
- Joint properties: parent/child display, value slider

### Status Bar (bottom)

`ui/status_bar.rs` — `TopBottomPanel::bottom("status_bar")` — Shows:
- Triangle count
- Selected node measurement (e.g. `"Box: 2.00×2.00×2.00"`)
- FPS counter (calculated over 10 frames)

### Sketch Mode Panels

When the sketch editor is active (`editor.active == true`), main panels are hidden and only the status bar remains visible (`app.rs:51-54`). Instead, sketch-specific UI appears:

- **Sketch Toolbar** (`sketch_editor.rs:270`): `TopBottomPanel::top("sketch_toolbar")` with tool selection (Select, Line, Circle, Arc, Measure), constraint visibility toggle, undo/redo, extrude controls, and Finish/Cancel buttons.
- **Sketch Constraints** (`sketch_editor.rs:365`): `SidePanel::right("sketch_constraints")` listing all constraints with context-menu deletion and inline value editing.

---

## AppState — Bridge Between Bevy and egui

`app.rs:10` — `AppState` is a Bevy `Resource` that all systems share. It carries:

```rust
pub struct AppState {
    pub document: Document,
    pub history: CommandHistory,
    pub mesh_gen: u64,
    pub pending_sketch_edit: Option<String>,
    pub clipboard: Option<GeometryNode>,
    pub show_array_dialog: bool,
    pub show_mirror_dialog: bool,
    pub show_fillet_dialog: bool,
    pub show_chamfer_dialog: bool,
    pub show_joint_dialog: bool,
    pub new_joint_type: JointType,
    pub new_joint_pivot: [f64; 3],
    pub new_joint_axis: [f64; 3],
    pub array_params: ArrayParams,
}
```

- **document**: The current `Document` (recipe + evaluated meshes + selection state).
- **history**: Undo/redo command stacks.
- **mesh_gen**: Monotonically increasing counter. Incremented by `mark_dirty()` to signal `sync_meshes()` that meshes need updating.
- **pending_sketch_edit**: When a user double-clicks a Sketch node in the scene tree, the node ID is stored here. `check_enter_sketch_mode()` picks this up on the next frame and initializes the sketch editor.
- **clipboard**: The last copied/cut node for paste operations.
- **Dialog booleans**: `show_*_dialog` flags toggle modal windows for array, mirror, fillet, chamfer, and joint creation.
- **Joint state**: `new_joint_*` fields hold the in-progress joint definition before creation.

---

## mark_dirty() Propagation

`app.rs:34` — Called after any document mutation to trigger re-evaluation and re-rendering:

```rust
pub fn mark_dirty(&mut self) {
    self.mesh_gen += 1;
    self.document.is_modified = true;
}
```

The frame loop then reacts:
1. `app::ui_system()` — UI panels may read/write state (e.g. property changes).
2. `document::evaluate_all()` — Called by commands that mutate the tree.
3. `sync::sync_meshes()` — Detects `mesh_gen != last_gen`, rebuilds Bevy meshes.
4. Window title update — Shows `*` when modified (`main.rs:268`).

---

## Scene Tree

`ui/scene_tree.rs:239` — `tree_node()` renders each node recursively:

- **Selection logic**: Single click sets `Document.selection` and clears `multi_selection`. Ctrl+click toggles into `multi_selection` without clearing.
- **Visibility toggle**: `[v]` / `[ ]` buttons that add/remove from `Document.hidden_nodes`. Hidden nodes are skipped during `sync_meshes()`.
- **Double-click sketch**: Sets `pending_sketch_edit` to enter sketch editing mode.
- **Context menu**: Right-click shows "Delete" option for non-root nodes.
- **Fillet/Chamfer children**: Special-cased — the child of a fillet/chamfer is displayed inline under the parent (not indented separately).

---

## Properties Panel

`ui/properties.rs:58` — `show_all_properties()` dispatches to type-specific editors:

- Primitive editors use `float_drag()` which captures old/new values and creates `SetParameterCommand`.
- Transform editing mutates the node's `TransformOp` directly (no command wrapping — changes are immediate and not undoable individually).
- Color editing uses egui's `color_edit_button_srgb`, converts to hex string, stores on the node.
- Joint properties show a slider for `current_value` using `SetJointValueCommand`.

---

## Dialog System

Dialogs are modal `egui::Window`s rendered conditionally in `scene_tree.rs`:

- **Array**: Configures count, translation offset, rotation offset, and scale multiplier. Calls `array_selected()` which creates `AddFeatureCommand` per copy.
- **Mirror**: Selects a mirror plane (XY, XZ, YZ). Calls `mirror_selected()` which creates a mirrored copy via negative scale and an `AddFeatureCommand`.
- **Fillet/Chamfer**: Same pattern — wraps the selected node in a Fillet/Chamfer parent using `AddFeatureCommand`, preceded by `DeleteFeatureCommand` to remove the original.
- **Joint**: Configures type, pivot, and axis. Calls `add_joint()` which creates an `AddJointCommand`.
