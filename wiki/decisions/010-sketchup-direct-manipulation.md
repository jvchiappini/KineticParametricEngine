# ADR-010 — SketchUp-Style Direct 3D Manipulation as Primary Workflow

**Status:** `accepted`
**Date:** 2026-06-02
**Author:** KPE Engineering Team

---

## Context

The original KPE architecture separated 2D sketch creation (on a sketch plane) from 3D extrusion, following a traditional parametric CAD workflow (sketch → feature → solid). While powerful for precise engineering, this workflow has a high interaction cost:

1. Users must enter sketch mode, draw, exit sketch mode, then extrude — multiple context switches.
2. The 2D sketch plane is abstract; users cannot directly see where their geometry will end up in 3D.
3. Rapid ideation and iterative exploration (common in furniture design) is slowed by the modal workflow.

SketchUp proved that a "direct manipulation" approach — draw in 3D, push-pull faces — dramatically lowers the barrier to entry while remaining powerful enough for production work. The challenge is integrating this immediacy with KPE's parametric core.

## Options Considered

### Option A — Pure 2D Sketch + Extrude (Existing)

The original design: every solid starts as a 2D sketch, then extrude/revolve/sweep features create 3D.

**Pros:**
- Fully parametric; every dimension is editable after creation
- Matches traditional CAD (SolidWorks, Fusion 360) mental model
- Existing implementation already functional

**Cons:**
- High context-switch cost (in/out of sketch mode)
- Cannot see 3D result while sketching
- Slower for furniture design rapid iteration
- Does not match user's SketchUp expectation

### Option B — Pure Direct Mesh (Blender/SketchUp)

Every tool creates/modifies mesh geometry directly. No parametric history.

**Pros:**
- Maximum fluidity — click, draw, push, pull
- Lowest learning curve
- Matches SketchUp exactly

**Cons:**
- Loses parametric editability — cannot change a dimension later
- Not compatible with KPE's parametric solver, rules engine, or fabrication pipeline
- Mesh-only geometry degrades with repeated boolean operations
- Violates KPE's core value proposition (parametric design)

### Option C — Hybrid: Direct 3D Tools → Parametric Nodes (Chosen)

Tool operations in the 3D viewport create parametric `GeometryNode` entries in the scene graph:

- **Rectangle tool** → creates `BoxDef { width, depth, height: 0.01 }` (thin slab)
- **Circle tool** → creates `CylinderDef { radius, height: 0.01 }` (thin disc)
- **Line tool** → creates thin box segments (future)
- **Push/Pull tool** → on Box/Cylinder faces: adjusts height parameter; on Mesh faces: extrudes mesh geometry
- **Select tool** → existing AABB node selection, gated by tool activation
- **Move tool** → translates node via transform (future)
- **Eraser tool** → deletes node via command (future)

All tool actions go through `AddFeatureCommand` / parameter commands, making them fully undoable and preserving the parametric scene graph.

**Pros:**
- SketchUp-like immediacy in the viewport
- All geometry is parametric nodes — dimensions editable after creation
- Fully undoable via existing CommandHistory
- No context switching; everything happens in the 3D viewport
- Existing 2D sketch mode remains available for complex profiles
- Construction planes inferred from face hits or ground plane (automatic)

**Cons:**
- Rectangle/Circle initially create zero-height primitives (hack); push-pull expected immediately after
- Some tool operations are non-parametric (mesh extrude)
- New surface needed: tool palette UI, construction plane inference
- Line tool polyline → box mapping is awkward for thin geometry

## Decision

**Option C was chosen.** The tool system (`build_tool` module) creates parametric nodes from direct 3D interactions, maintaining full editability while providing SketchUp-like fluidity.

### Key Design Decisions within Option C

1. **Construction plane inference**: First click ray-picks existing geometry; the hit face normal becomes the construction plane normal. Fallback: ground plane (Y=0).
2. **Tool gating**: Only the active tool's system responds to viewport clicks. `viewport_selection` is gated by Select tool; `face_pick_system` by PushPull tool.
3. **Parametric push-pull**: Future enhancement — when PushPull tool acts on a Box face, the BoxDef.height parameter is adjusted rather than extruding mesh.
4. **Scene graph integration**: All created nodes use `next_counter()` for unique IDs and `AddFeatureCommand` for full undo support.
5. **2D sketch coexistence**: The existing sketch editor remains available for complex parametric profiles. BuildTool is the default workflow.

## Consequences

**Positive:**
- Dramatically faster iteration for furniture design
- No context switching between 2D and 3D for basic shapes
- All operations are undoable via CommandHistory
- Tool palette is extensible (future: Offset, Trim, Follow Me)
- Construction plane inference makes drawing contextual

**Negative / Trade-offs:**
- Zero-height primitives (h=0.01) are a visual kludge until push-pulled
- Mesh extrude (existing `extrude_face()`) is non-parametric
- Line tool will create thin boxes rather than true edges (kpe-schema has no edge primitive)

**Neutral:**
- The 2D sketch mode persists as an advanced feature for complex profiles
- Push-pull on parametric primitives requires future work (height adjustment vs mesh extrude)

## References

- ADR-006: Sketch + Extrude Pipeline (original approach)
- ADR-007: Advanced Sketch Engine (DCEL topology context)
- `apps/desktop/src/build_tool/mod.rs` — BuildTool implementation
- `apps/desktop/src/push_pull_system.rs` — face-level ray-triangle picking + extrusion
- SketchUp direct manipulation paradigm (industry reference)
