use bevy::prelude::*;
use bevy::tasks::Task;
use crate::{app::AppState, sketch_editor};
use kpe_parametric::commands::SetSketchCommand;
use kpe_geometry::sketch::document::SketchDocument;
use sketch_editor::solver::SolveTask;

pub struct SketchEditorPlugin;

impl Plugin for SketchEditorPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(sketch_editor::SketchEditorState::new())
            .insert_resource(SolveTask::default())
            .add_systems(Update, sketch_editor::check_enter_sketch_mode)
            .add_systems(Update, sketch_editor::sketch_input)
            .add_systems(Update, sketch_editor::render_sketch)
            .add_systems(Update, sketch_editor::render_sketch_wireframes)
            .add_systems(Update, sketch_editor::sketch_ui)
            .add_systems(Update, handle_pending_actions)
            .add_systems(Update, poll_solve_task);
    }
}

fn handle_pending_actions(
    mut editor: ResMut<sketch_editor::SketchEditorState>,
    mut state: ResMut<AppState>,
) {
    if let Some(ext) = editor.pending_extrude.take() {
        let node_id = editor.node_id.clone();
        if let Some(mut old_sketch) = sketch_editor::math::get_sketch_def_local(
            &state.document.recipe.scene, &node_id)
        {
            old_sketch.extrude = Some(kpe_schema::geometry::ExtrudeDef {
                sketch_id: node_id.clone(),
                distance: ext.distance,
                direction: None,
                cap: true,
                taper_angle: if ext.taper_angle != 0.0 { Some(ext.taper_angle) } else { None },
            });
            state.execute(Box::new(SetSketchCommand {
                node_id: node_id.clone(),
                old_sketch: None,
                new_sketch: old_sketch,
            }));
        }
    }

    if editor.pending_finish {
        editor.pending_finish = false;
        if let Some((node_id, sketch_def)) = editor.exit() {
            state.execute(Box::new(SetSketchCommand {
                node_id: node_id.clone(),
                old_sketch: None,
                new_sketch: sketch_def,
            }));
        }
    }

    if editor.pending_cancel {
        editor.pending_cancel = false;
        editor.active = false;
        editor.document = SketchDocument::new();
    }
}

fn poll_solve_task(
    mut solve_task: ResMut<SolveTask>,
    mut editor: ResMut<sketch_editor::SketchEditorState>,
) {
    if let Some(task) = &mut solve_task.0 {
        if let Some(result) = poll_task(task) {
            editor.document = result.document;
            if let Some(err) = result.error {
                editor.last_solve_error = Some(err);
            }
            solve_task.0 = None;
        }
    }
}

fn poll_task<T>(task: &mut Task<T>) -> Option<T> {
    use bevy::tasks::futures_lite::FutureExt;
    let mut pinned = std::pin::pin!(task);
    match pinned.as_mut().poll(&mut std::task::Context::from_waker(
        &noop_waker(),
    )) {
        std::task::Poll::Ready(val) => Some(val),
        std::task::Poll::Pending => None,
    }
}

fn noop_waker() -> std::task::Waker {
    std::task::Waker::noop().clone()
}
