# Selection System

## Selection State

`apps/desktop/src/document.rs:35-38` — The `Document` struct holds three selection-related fields:

```rust
pub selection: Option<String>,
pub multi_selection: Vec<String>,
pub joint_selection: Option<String>,
```

- **selection**: The primary (last-clicked) node ID. Used by most operations (properties panel, deletion, transforms, gizmos).
- **multi_selection**: Secondary selection for bulk operations (delete, copy, cut). Populated via Ctrl+click.
- **joint_selection**: Separate selection track for joints in the joint list. When set, the properties panel shows joint controls instead of node properties.

## Scene Tree Selection

`ui/scene_tree.rs:256-271` — Click behavior in the tree:

- **Single click** without Ctrl: Sets `selection = Some(node.id)` and clears `multi_selection`.
- **Ctrl+click**: Toggles the node ID in `multi_selection`. If the node was already the primary selection, it clears `selection` too.
- **Double-click on a Sketch node**: Sets `pending_sketch_edit` (not a selection action).

### Visibility Toggles

`scene_tree.rs:245-253` — Each tree node has a `[v]` / `[ ]` button that adds/removes the node ID from `Document.hidden_nodes`. Hidden nodes are excluded from ray-picking and mesh rendering.

## 3D Viewport Selection

`main.rs:138-209` — Ray-AABB picking in the 3D viewpoint. The `viewport_selection` system runs on every left-click:

1. **Prerequisites**: Left button just pressed, RMB and MMB not pressed, cursor within the viewport area (not over panels), sketch editor not active.
2. **Ray casting**: Uses `Camera::viewport_to_world()` to get a world-space ray from the cursor position.
3. **Ray-AABB test**: For every entity with `MeshNodeId` and `Aabb` components:
   - Transform the ray from world space to model (local) space using the entity's inverse world matrix.
   - Perform slab-based AABB intersection in local space.
   - Track the closest hit by distance.
4. **Selection update**: If a hit is found:
   - Without Ctrl: `selection = Some(id)`.
   - With Ctrl: If already selected, deselect; otherwise select.

### MeshNodeId Component

`sync.rs:28` — A simple component linking each spawned mesh entity to its scene node:

```rust
#[derive(Component)]
pub struct MeshNodeId(pub String);
```

This is attached to every child entity under `SceneMeshRoot` during `sync_meshes()`. The value is the `GeometryNode.id` string, so intersecting a ray with an entity immediately yields the corresponding node.

## Effects of Selection

### Properties Panel

`ui/properties.rs:25-39` — When `Document.selection` is `Some(id)`, the properties panel looks up the node, displays its type-specific parameters, transform editors, and color picker. If `joint_selection` is set, joint properties are shown instead.

### Deletion

`main.rs:72-75` (keyboard) and `feature_commands.rs:187-221` — Delete/Backspace removes the selected node(s). `delete_selected_nodes()` collects from both `selection` and `multi_selection`, creates `DeleteFeatureCommand` for each, and clears all selection state.

### Transform / Gizmo

`gizmos.rs` — The gizmo system reads `Document.selection` to determine which node to draw translation arrows for. Only single selection is supported for gizmo interaction.

### Copy, Cut, Paste, Duplicate

`commands.rs:252-347` — All clipboard operations depend on `Document.selection` to identify the source node. Cut/Paste also interact with `multi_selection` indirectly via the delete path.

### Array, Mirror, Fillet, Chamfer

`feature_commands.rs` — These operations use `selection` to identify the node to duplicate or wrap in a fillet/chamfer parent.
