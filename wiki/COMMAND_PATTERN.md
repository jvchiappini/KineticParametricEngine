# Command Pattern (Undo/Redo System)

## Command Trait

`apps/desktop/src/commands.rs:5` — All undoable operations implement the `Command` trait:

```rust
pub trait Command: Send + Sync {
    fn execute(&mut self, doc: &mut Document);
    fn undo(&mut self, doc: &mut Document);
    fn description(&self) -> &str;
}
```

- **execute**: Applies the mutation to the document.
- **undo**: Reverses the mutation, restoring the document to its prior state.
- **description**: Human-readable label (displayed in tooltip or menus).

The `Send + Sync` bounds allow commands to be stored in `Vec<Box<dyn Command>>`.

## Command Types

### SetParameterCommand

`commands.rs:11` — Changes a single numeric parameter on a primitive node. Stores both old and new values. Used when the user drags a dimension in the Properties panel (`properties.rs:273`).

- **execute**: Calls `apply_param()` to write the new value, then calls `evaluate_node()` to rebuild only that node's mesh.
- **undo**: Restores the old value and re-evaluates.

### AddFeatureCommand

`commands.rs:79` — Inserts a new `GeometryNode` as a child of a specified parent. Used by all primitive creation functions (`add_box`, `add_cylinder`, `add_sketch`, etc.) and by array/mirror operations.

- **execute**: Appends the node to the parent's children, then calls `evaluate_all()`.
- **undo**: Calls `remove_child()` to detach the node by ID, then re-evaluates everything.

### DeleteFeatureCommand

`commands.rs:117` — Removes a node from the tree. Store the parent ID and a clone of the removed node for undo.

- **execute**: Removes the node, clears selection if it pointed to this node, re-evaluates.
- **undo**: Re-inserts the saved node clone under the saved parent, re-evaluates.

### AddJointCommand

`commands.rs:349` — Appends a new `Joint` to the recipe's joint list.

- **execute**: Pushes the joint, re-evaluates (joints affect world matrices).
- **undo**: Retains the joint by ID, re-evaluates.

### SetJointValueCommand

`commands.rs:369` — Changes `current_value` on an existing joint (e.g. dragging the angle slider).

- **execute**: Mutates the in-place joint, re-evaluates.
- **undo**: Restores the old value, re-evaluates.

### SaveCommand

`io.rs:176` — A special command whose `undo` is a no-op. Used for save-as operations that don't mutate document state.

## CommandHistory

`commands.rs:141` — Manages undo/redo stacks with a bounded capacity.

```rust
pub struct CommandHistory {
    pub undo_stack: Vec<Box<dyn Command>>,
    pub redo_stack: Vec<Box<dyn Command>>,
    pub max_undo: usize,
}
```

- **max_undo**: Hard cap of 50 commands (`commands.rs:149`). When exceeded, the oldest undo is dropped from the front of the stack.

### execute(cmd, doc)

`commands.rs:152` — The primary entry point for running a command:

1. Call `cmd.execute(doc)` — apply the mutation.
2. Push `cmd` onto `undo_stack`.
3. **Clear `redo_stack`** — any new action invalidates the redo history.
4. If `undo_stack.len() > max_undo`, remove the oldest entry.

### undo(doc)

`commands.rs:162` — Pop the top of `undo_stack`, call `cmd.undo(doc)`, push onto `redo_stack`.

### redo(doc)

`commands.rs:169` — Pop the top of `redo_stack`, call `cmd.execute(doc)`, push onto `undo_stack`.

### can_undo / can_redo

Simple boolean checks on stack emptiness. Used to enable/disable UI buttons.

## Integration

- **Toolbar**: Undo/Redo buttons call `history.undo()` and `history.redo()` (`toolbar.rs:38-48`).
- **Keyboard**: `Ctrl+Z` undoes, `Ctrl+Shift+Z` or `Ctrl+Y` redoes (`main.rs:79-95`).
- **Properties panel**: Dimension drags create `SetParameterCommand` via `history.execute()` (`properties.rs:273`).
- **Feature creation**: All `add_*` functions in `feature_commands.rs` wrap operations in `AddFeatureCommand`.

## What Is NOT Undoable

- **Sketch edits**: The sketch editor (`sketch_editor.rs`) maintains its own independent undo stack of `SketchDocument` snapshots, not using the `Command` trait. Sketch edits are in-memory and do not hit the document until "Finish" is clicked.
- **Gizmo dragging**: Direct mutation via `apply_translation()` (`gizmos.rs:204`) with `evaluate_node()` — no command wrapping.
- **Viewport operations**: Camera movement, selection changes, and visibility toggles are not recorded.
