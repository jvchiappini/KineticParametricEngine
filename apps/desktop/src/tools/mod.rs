//! SketchUp‑style tool system for KPE.
//!
//! Each tool lives in its own module and is registered in `main.rs` as
//! Bevy systems.  Shared infrastructure lives in `core`.

pub mod core;
pub mod push_pull;
pub mod select;
pub mod rectangle;
pub mod circle;
pub mod line;
pub mod move_tool;
pub mod eraser;

// Re‑export common items for convenience.
pub use core::{BuildTool, BuildToolState, ConstructionPlane, ToolPhase};
pub use push_pull::PushPullToolState;
