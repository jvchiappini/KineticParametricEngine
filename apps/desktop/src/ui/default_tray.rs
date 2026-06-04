use bevy_egui::{egui, EguiContexts};
use crate::app::AppState;
use crate::ui::properties;


pub fn show(contexts: &mut EguiContexts, state: &mut AppState) {
    let ctx = contexts.ctx_mut();

    egui::SidePanel::right("default_tray")
        .resizable(true)
        .default_width(320.0)
        .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(0.0).fill(egui::Color32::from_rgb(38, 38, 38)))
        .show(ctx, |ui| {
            // Default Tray header
            let header_frame = egui::Frame::none()
                .fill(egui::Color32::from_rgb(50, 50, 50))
                .inner_margin(egui::Margin::same(6.0));
            
            header_frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Default Tray").strong().color(egui::Color32::from_rgb(220, 220, 220)));
                });
            });

            // Make the scrollable area for the tray items
            egui::ScrollArea::vertical().id_salt("default_tray_scroll").show(ui, |ui| {
                let section_frame = egui::Frame::none()
                    .fill(egui::Color32::from_rgb(45, 45, 45))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)))
                    .inner_margin(egui::Margin::same(6.0));
                
                // Scene Tree Panel
                section_frame.show(ui, |ui| {
                    egui::CollapsingHeader::new(egui::RichText::new("Scene (Outliner)").strong().color(egui::Color32::from_rgb(220, 220, 220)))
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.add_space(4.0);
                            crate::ui::scene_tree::show_content(ui, state);
                            ui.add_space(4.0);
                        });
                });

                // Entity Info Panel
                section_frame.show(ui, |ui| {
                    egui::CollapsingHeader::new(egui::RichText::new("Entity Info").strong().color(egui::Color32::from_rgb(220, 220, 220)))
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.add_space(4.0);
                            properties::show_entity_info_content(ui, state);
                            ui.add_space(4.0);
                        });
                });

                // Materials Panel
                section_frame.show(ui, |ui| {
                    egui::CollapsingHeader::new(egui::RichText::new("Materials").strong().color(egui::Color32::from_rgb(220, 220, 220)))
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new("In Model").italics().color(egui::Color32::from_rgb(180, 180, 180)));
                            ui.separator();
                            ui.label("Default Material");
                            ui.label("Color #A134");
                            ui.label("Glass");
                            ui.label("Wood");
                            ui.add_space(4.0);
                        });
                });

                // Components Panel
                section_frame.show(ui, |ui| {
                    egui::CollapsingHeader::new(egui::RichText::new("Components").strong().color(egui::Color32::from_rgb(220, 220, 220)))
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new("In Model").italics().color(egui::Color32::from_rgb(180, 180, 180)));
                            ui.separator();
                            ui.label("No components in model.");
                            ui.add_space(4.0);
                        });
                });

                // Styles Panel
                section_frame.show(ui, |ui| {
                    egui::CollapsingHeader::new(egui::RichText::new("Styles").strong().color(egui::Color32::from_rgb(220, 220, 220)))
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.add_space(4.0);
                            ui.label("Default Style");
                            ui.add_space(4.0);
                        });
                });

                // Tags Panel
                section_frame.show(ui, |ui| {
                    egui::CollapsingHeader::new(egui::RichText::new("Tags").strong().color(egui::Color32::from_rgb(220, 220, 220)))
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.add_space(4.0);
                            ui.label("Untagged");
                            ui.add_space(4.0);
                        });
                });
            });
        });
}
