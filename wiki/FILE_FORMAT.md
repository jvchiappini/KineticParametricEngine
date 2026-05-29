# File Format

## .kpe Document Format

`apps/desktop/src/io.rs` — The primary file format is JSON, serialized via serde. Files use the `.kpe` extension.

### Structure

The top-level structure is `KPERecipe` (`crates/kpe-schema/src/recipe.rs:9`):

```json
{
    "version": "0.1.0",
    "metadata": {
        "name": "Untitled",
        "author": null,
        "description": null,
        "created_at": null,
        "tags": []
    },
    "blocks": {},
    "scene": {
        "id": "root",
        "node_type": "Compound",
        "transform": null,
        "children": [ /* ... recursive GeometryNode tree ... */ ],
        "operations": [],
        "color": null
    },
    "joints": [ /* ... Joint definitions ... */ ],
    "constraints": [],
    "materials": {},
    "precision": 1e-6
}
```

All numeric values are `f64` (64-bit floats). The JSON is produced with `serde_json::to_string_pretty()` for human readability.

### GeometryNode serialization

`GeometryNodeType` is serialized as a tagged enum. For example:
```json
{
    "id": "Box_001",
    "node_type": {
        "Box": { "width": 2.0, "height": 2.0, "depth": 2.0 }
    },
    "transform": {
        "translation": [1.0, 0.0, 0.0],
        "rotation": null,
        "scale": null
    },
    "children": [],
    "operations": [],
    "color": "#FF8800"
}
```

### Joint serialization

```json
{
    "id": "Joint_1",
    "joint_type": "Revolute",
    "parent_id": "Root",
    "child_id": "Box_001",
    "pivot": [0.0, 0.0, 0.0],
    "axis": [0.0, 1.0, 0.0],
    "limits": { "min": -180.0, "max": 180.0, "damping": null, "stiffness": null },
    "current_value": 0.0
}
```

## Save / Load Workflow

### Save

`io.rs:34` — `save_to_file()` serializes `doc.recipe` as pretty-printed JSON and writes to the chosen path. The user selects the file via a native dialog (`rfd::FileDialog`). The document's `file_path` is updated to the saved location.

### Open

`io.rs:42` — `load_from_file()` reads the file, deserializes via `serde_json::from_str()`, creates a new `Document` with the loaded recipe, calls `evaluate_all()` to build all meshes, and sets the initial selection to the first node ID.

### Dialog Functions

`io.rs:6-32` — Three dialog types:
- `save_dialog()` — Filter: `.kpe`, default name: `untitled.kpe`
- `open_dialog()` — Filter: `.kpe`
- `export_stl_dialog()` / `export_obj_dialog()` — Separate file dialogs for export

## Export Formats

### STL Binary

`io.rs:54` — Binary STL export (most common for 3D printing):
- 80-byte ASCII header (contains "KPE Export")
- 4-byte unsigned 32-bit triangle count (little-endian)
- For each triangle: 12 bytes of normal (3×f32 LE) + 36 bytes of vertices (3×3×f32 LE) + 2-byte attribute count (always 0)

Total of 50 bytes per triangle + 84 bytes header. Triangles are collected from all meshes in `doc.evaluated.meshes`. If total exceeds `u32::MAX`, export is rejected.

### OBJ Wavefront

`io.rs:108` — Text-based OBJ export:
- `# KPE Export` header with triangle count
- Each mesh is exported as a separate object (`o {node_id}`)
- Vertex positions (`v x y z`), normals (`vn x y z`), and face indices (`f a b c`)
- Vertex offsets are accumulated across meshes so indices remain valid

## Auto-Save

`main.rs:216-234` — An auto-save system runs every 120 seconds (2 minutes):

```rust
#[derive(Resource)]
struct AutoSaveTimer(Timer);

fn auto_save_system(time: Res<Time>, mut timer: ResMut<AutoSaveTimer>, state: Res<app::AppState>) {
    if timer.0.tick(time.delta()).just_finished() {
        let path = dirs_data_local().join("kpe_autosave.kpe");
        // writes serde_json::to_string(&state.document.recipe)
    }
}
```

The auto-save file is located at `%APPDATA%/KPE/kpe_autosave.kpe` on Windows (falls back to `.kpe/kpe_autosave.kpe` if `APPDATA` is not set). The directory is created if it does not exist. The timer is a repeating `Timer` with `TimerMode::Repeating`.
