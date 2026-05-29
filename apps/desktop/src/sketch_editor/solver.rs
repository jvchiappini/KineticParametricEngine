use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use kpe_geometry::sketch::document::SketchDocument;
use kpe_geometry::sketch::analyze_dof;
use super::state::SketchEditorState;

#[derive(Resource)]
pub struct SolveTask(pub Option<Task<SolveResult>>);

pub struct SolveResult {
    pub document: SketchDocument,
    pub error: Option<String>,
}

impl Default for SolveTask {
    fn default() -> Self { Self(None) }
}

pub fn request_solve(editor: &mut SketchEditorState, solve_task: &mut SolveTask) {
    let doc = editor.document.clone();
    editor.last_solve_error = None;

    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(async move {
        let mut cloned = doc;
        let error = match cloned.solve() {
            Ok(()) => None,
            Err(e) => Some(format!("Solver: {}", e)),
        };
        SolveResult { document: cloned, error }
    });

    solve_task.0 = Some(task);
}

/// Run solve synchronously, capture errors, and update DoF analysis.
pub fn solve_sync(editor: &mut SketchEditorState) {
    editor.last_solve_error = None;
    if let Err(e) = editor.document.solve() {
        editor.last_solve_error = Some(format!("Solver: {}", e));
    }
    editor.dof_status = analyze_dof(
        &editor.document.points,
        &editor.document.lines,
        &editor.document.arcs,
        &editor.document.circles,
        &editor.document.constraints,
    );
}
