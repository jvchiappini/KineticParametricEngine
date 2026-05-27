use bevy_egui::{egui, EguiContexts};
use crate::app::AppState;

pub fn show(contexts: &mut EguiContexts, state: &mut AppState) {
    egui::TopBottomPanel::top("toolbar")
        .min_height(32.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                ui.heading("KPE");
                ui.separator();

                if ui.button("New").clicked() {
                    let mut doc = crate::document::Document::new();
                    doc.recipe.scene = crate::app::default_scene();
                    doc.evaluate_all();
                    doc.selection = Some("Box_001".to_string());
                    state.document = doc;
                    state.history = crate::commands::CommandHistory::new();
                    state.mark_dirty();
                }

                if ui.button("Open").clicked() {
                    if let Some(doc) = crate::io::open_document() {
                        state.document = doc;
                        state.history = crate::commands::CommandHistory::new();
                        state.mark_dirty();
                    }
                }

                if ui.button("Save").clicked() {
                    let path = crate::io::save_dialog(&state.document);
                    if let Some(p) = path {
                        let _ = crate::io::save_to_file(&p, &state.document);
                    }
                }

                ui.separator();

                let can_undo = state.history.can_undo();
                let undo = ui.add_enabled(can_undo, egui::Button::new("Undo"));
                if undo.clicked() {
                    state.history.undo(&mut state.document);
                    state.mark_dirty();
                }

                let can_redo = state.history.can_redo();
                let redo = ui.add_enabled(can_redo, egui::Button::new("Redo"));
                if redo.clicked() {
                    state.history.redo(&mut state.document);
                    state.mark_dirty();
                }

                ui.separator();

                ui.menu_button("Export", |ui| {
                    if ui.button("STL (Binary)").clicked() {
                        if let Some(path) = crate::io::export_stl_dialog() {
                            let _ = crate::io::export_stl(&path, &state.document);
                        }
                        ui.close_menu();
                    }
                    if ui.button("OBJ (Wavefront)").clicked() {
                        if let Some(path) = crate::io::export_obj_dialog() {
                            let _ = crate::io::export_obj(&path, &state.document);
                        }
                        ui.close_menu();
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let tri_count = state.document.evaluated.triangle_count();
                    ui.label(format!("Tris: {}", tri_count));
                });
            });
        });
}
