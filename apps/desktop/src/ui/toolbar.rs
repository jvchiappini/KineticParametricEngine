use bevy_egui::{egui, EguiContexts};
use crate::app::AppState;

pub fn show(contexts: &mut EguiContexts, state: &mut AppState) {
    let ctx = contexts.ctx_mut();
    egui::TopBottomPanel::top("toolbar")
        .min_height(36.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("KPE");
                ui.separator();

                if ui.button("New").on_hover_text("Create a new empty document").clicked() {
                    state.document = crate::document::Document::new();
                    state.history = kpe_parametric::CommandHistory::new();
                    state.mark_dirty();
                }

                if ui.button("Open").on_hover_text("Open a .kpe file (Ctrl+O)").clicked() {
                    if let Some(doc) = crate::io::open_document() {
                        state.document = doc;
                        state.history = kpe_parametric::CommandHistory::new();
                        state.mark_dirty();
                    }
                }

                if ui.button("Save").on_hover_text("Save document (Ctrl+S)").clicked() {
                    let path = state.document.file_path.clone()
                        .map(|p| std::path::PathBuf::from(p))
                        .or_else(|| crate::io::save_dialog(&state.document));
                    if let Some(ref p) = path {
                        if crate::io::save_to_file(p, &state.document).is_ok() {
                            state.document.file_path = Some(p.to_string_lossy().to_string());
                            state.document.is_modified = false;
                        }
                    }
                }

                ui.separator();

                let can_undo = state.history.can_undo();
                let undo = ui.add_enabled(can_undo, egui::Button::new("Undo  Ctrl+Z"))
                    .on_hover_text("Undo last operation");
                if undo.clicked() {
                    state.undo();
                }

                let can_redo = state.history.can_redo();
                let redo = ui.add_enabled(can_redo, egui::Button::new("Redo  Ctrl+Shift+Z"))
                    .on_hover_text("Redo previously undone operation");
                if redo.clicked() {
                    state.redo();
                }

                ui.separator();

                ui.menu_button("Export", |ui| {
                    if ui.button("STL (Binary)").on_hover_text("Export as binary STL for 3D printing").clicked() {
                        if let Some(path) = crate::io::export_stl_dialog() {
                            let _ = crate::io::export_stl(&path, &state.document);
                        }
                        ui.close_menu();
                    }
                    if ui.button("OBJ (Wavefront)").on_hover_text("Export as Wavefront OBJ").clicked() {
                        if let Some(path) = crate::io::export_obj_dialog() {
                            let _ = crate::io::export_obj(&path, &state.document);
                        }
                        ui.close_menu();
                    }
                    if ui.button("DXF (2D)").on_hover_text("Export as 2D DXF for CNC/laser cutting").clicked() {
                        if let Some(path) = crate::io::export_dxf_dialog() {
                            let _ = crate::io::export_dxf(&path, &state.document);
                        }
                        ui.close_menu();
                    }
                    if ui.button("SVG (2D)").on_hover_text("Export as 2D SVG for laser cutting").clicked() {
                        if let Some(path) = crate::io::export_svg_dialog() {
                            let _ = crate::io::export_svg(&path, &state.document);
                        }
                        ui.close_menu();
                    }
                });

                ui.separator();
                if ui.button("Front").on_hover_text("Front view (key 1)").clicked() {
                    state.pending_view_preset = Some(1);
                }
                if ui.button("Top").on_hover_text("Top view (key 2)").clicked() {
                    state.pending_view_preset = Some(2);
                }
                if ui.button("Right").on_hover_text("Right view (key 3)").clicked() {
                    state.pending_view_preset = Some(3);
                }
                if ui.button("Iso").on_hover_text("Isometric view (key 4)").clicked() {
                    state.pending_view_preset = Some(4);
                }

                ui.separator();
                if ui.button("?").on_hover_text("About KPE").clicked() {
                    state.show_about_dialog = true;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let tri_count = state.document.evaluated.triangle_count();
                    ui.label(format!("Tris: {}", tri_count));
                });
            });
        });
}
