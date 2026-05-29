use crate::document::Document;
use crate::ui;
use crate::sketch_editor::SketchEditorState;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use kpe_parametric::CommandHistory;
use kpe_parametric::commands::features::ArrayParams;
use kpe_schema::geometry::GeometryNode;
use kpe_schema::joint::JointType;

#[derive(Resource)]
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
    pub show_about_dialog: bool,
    pub new_joint_type: JointType,
    pub new_joint_pivot: [f64; 3],
    pub new_joint_axis: [f64; 3],
    pub array_params: ArrayParams,
    pub pending_view_preset: Option<u8>,
}

impl AppState {
    pub fn new() -> Self {
        let doc = Document::new();
        Self {
            document: doc,
            history: CommandHistory::new(),
            mesh_gen: 1,
            pending_sketch_edit: None,
            clipboard: None,
            show_array_dialog: false,
            show_mirror_dialog: false,
            show_fillet_dialog: false,
            show_chamfer_dialog: false,
            show_joint_dialog: false,
            show_about_dialog: false,
            new_joint_type: JointType::Revolute,
            new_joint_pivot: [0.0; 3],
            new_joint_axis: [0.0, 1.0, 0.0],
            array_params: ArrayParams::default(),
            pending_view_preset: None,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.mesh_gen += 1;
        self.document.is_modified = true;
    }

    /// Undo the last command, updating both scene and geometry.
    pub fn undo(&mut self) {
        let mut gs = self.document.to_scene();
        self.history.undo(&mut gs);
        self.document.apply_scene(gs);
        self.mark_dirty();
    }

    /// Redo the last undone command.
    pub fn redo(&mut self) {
        let mut gs = self.document.to_scene();
        self.history.redo(&mut gs);
        self.document.apply_scene(gs);
        self.mark_dirty();
    }

    /// Execute a parametric command, updating both scene and geometry.
    pub fn execute(&mut self, cmd: Box<dyn kpe_parametric::Command>) {
        let mut gs = self.document.to_scene();
        self.history.execute(cmd, &mut gs);
        self.document.apply_scene(gs);
        self.mark_dirty();
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn ui_system(
    mut contexts: EguiContexts,
    mut state: ResMut<AppState>,
    editor: Res<SketchEditorState>,
) {
    // Hide main panels during sketch editing to avoid layout flicker
    if editor.active {
        ui::status_bar::show(&mut contexts, &mut *state);
        return;
    }
    ui::toolbar::show(&mut contexts, &mut *state);
    ui::scene_tree::show(&mut contexts, &mut *state);
    ui::properties::show(&mut contexts, &mut *state);

    if state.show_about_dialog {
        let ctx = contexts.ctx_mut();
        egui::Window::new("About KPE")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("Kinetic Parametric Engine");
                ui.label("Version 0.1.0");
                ui.separator();
                ui.label("A parametric CAD application for furniture design and fabrication.");
                ui.label("Built with Bevy 0.15 + egui 0.30 + Rust.");
                ui.separator();
                ui.label("Geometry kernel: manifold-csg + csgrs");
                ui.label("Export: STL, OBJ, DXF, SVG");
                ui.separator();
                if ui.button("Close").clicked() {
                    state.show_about_dialog = false;
                }
            });
    }

    ui::status_bar::show(&mut contexts, &mut *state);
}
