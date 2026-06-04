use std::time::Instant;
use bevy_egui::{egui, EguiContexts};
use crate::app::AppState;
use kpe_schema::geometry::{GeometryNode, GeometryNodeType};
use crate::commands;
use crate::units;

#[derive(Clone)]
pub struct FrameTimer {
    pub last: Instant,
    pub fps: f64,
    pub frame_count: u32,
}

impl Default for FrameTimer {
    fn default() -> Self {
        Self { last: Instant::now(), fps: 0.0, frame_count: 0 }
    }
}

fn node_measurement(node: &GeometryNode) -> Option<String> {
    match &node.node_type {
        GeometryNodeType::Box(b) => {
            Some(format!(
                "Box: {} × {} × {}  mm",
                units::fmt_compact(b.width),
                units::fmt_compact(b.height),
                units::fmt_compact(b.depth)
            ))
        }
        GeometryNodeType::Cylinder(c) => {
            Some(format!(
                "Cyl: r{} × h{}  mm",
                units::fmt_compact(c.radius),
                units::fmt_compact(c.height)
            ))
        }
        GeometryNodeType::Sphere(s) => {
            Some(format!("Sphere: r{}  mm", units::fmt_compact(s.radius)))
        }
        _ => None,
    }
}

pub fn show(contexts: &mut EguiContexts, state: &mut AppState) {
    egui::TopBottomPanel::bottom("status_bar")
        .min_height(28.0)
        .frame(egui::Frame::none()
            .fill(egui::Color32::from_rgb(26, 26, 26))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(50)))
            .inner_margin(egui::Margin::symmetric(6.0, 4.0)))
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                ui.set_min_height(20.0);

                // ── Left section: tri count + selection info ──────────────
                let tri_count = state.document.evaluated.triangle_count();
                ui.label(egui::RichText::new(format!("▲ {}", tri_count)).color(egui::Color32::from_gray(170)).size(12.0));

                ui.separator();

                if let Some(ref sel) = state.document.selection {
                    if let Some(node) = commands::find_node(&state.document.recipe.scene, sel) {
                        if let Some(meas) = node_measurement(node) {
                            ui.label(egui::RichText::new(meas).color(egui::Color32::from_gray(200)).size(12.0));
                            ui.separator();
                        }
                    }
                }

                // ── FPS ──────────────────────────────────────────────────
                let tid = egui::Id::new("frame_timer");
                if let Some(mut timer) = ui.ctx().data_mut(|d| d.get_temp::<FrameTimer>(tid)) {
                    timer.frame_count += 1;
                    if timer.frame_count >= 10 {
                        let now = Instant::now();
                        let elapsed = (now - timer.last).as_secs_f64();
                        if elapsed > 0.0 {
                            timer.fps = timer.frame_count as f64 / elapsed;
                        }
                        timer.last = now;
                        timer.frame_count = 0;
                    }
                    ui.label(egui::RichText::new(format!("{:.0} fps", timer.fps)).color(egui::Color32::from_gray(140)).size(12.0));
                    ui.ctx().data_mut(|d| d.insert_temp(tid, timer));
                } else {
                    ui.ctx().data_mut(|d| d.insert_temp(tid, FrameTimer::default()));
                }

                // ── Right section: OSnap toggles (AutoCAD style) ─────────
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Small label before OSnap buttons
                    ui.label(egui::RichText::new("OSNAP").color(egui::Color32::from_gray(110)).size(11.0));
                    ui.separator();

                    // Helper closure for toggle buttons
                    let snaps = &mut state.snaps;

                    let axes_active = snaps.axes;
                    let btn_axes = egui::Button::new(
                        egui::RichText::new("AXIS")
                            .size(11.0)
                            .color(if axes_active { egui::Color32::from_rgb(100, 200, 255) } else { egui::Color32::from_gray(130) })
                    )
                    .fill(if axes_active { egui::Color32::from_rgb(30, 60, 100) } else { egui::Color32::from_gray(38) })
                    .stroke(egui::Stroke::new(1.0, if axes_active { egui::Color32::from_rgb(80, 160, 255) } else { egui::Color32::from_gray(60) }))
                    .rounding(3.0)
                    .min_size(egui::vec2(42.0, 18.0));
                    if ui.add(btn_axes).on_hover_text("Snap to world axes (X/Y/Z inference)").clicked() {
                        snaps.axes = !snaps.axes;
                    }

                    let center_active = snaps.center;
                    let btn_center = egui::Button::new(
                        egui::RichText::new("CEN")
                            .size(11.0)
                            .color(if center_active { egui::Color32::from_rgb(100, 200, 100) } else { egui::Color32::from_gray(130) })
                    )
                    .fill(if center_active { egui::Color32::from_rgb(30, 70, 30) } else { egui::Color32::from_gray(38) })
                    .stroke(egui::Stroke::new(1.0, if center_active { egui::Color32::from_rgb(80, 200, 80) } else { egui::Color32::from_gray(60) }))
                    .rounding(3.0)
                    .min_size(egui::vec2(42.0, 18.0));
                    if ui.add(btn_center).on_hover_text("Snap to center of faces").clicked() {
                        snaps.center = !snaps.center;
                    }

                    let mid_active = snaps.midpoint;
                    let btn_mid = egui::Button::new(
                        egui::RichText::new("MID")
                            .size(11.0)
                            .color(if mid_active { egui::Color32::from_rgb(255, 200, 80) } else { egui::Color32::from_gray(130) })
                    )
                    .fill(if mid_active { egui::Color32::from_rgb(70, 55, 20) } else { egui::Color32::from_gray(38) })
                    .stroke(egui::Stroke::new(1.0, if mid_active { egui::Color32::from_rgb(255, 180, 60) } else { egui::Color32::from_gray(60) }))
                    .rounding(3.0)
                    .min_size(egui::vec2(42.0, 18.0));
                    if ui.add(btn_mid).on_hover_text("Snap to midpoints").clicked() {
                        snaps.midpoint = !snaps.midpoint;
                    }

                    let end_active = snaps.endpoint;
                    let btn_end = egui::Button::new(
                        egui::RichText::new("END")
                            .size(11.0)
                            .color(if end_active { egui::Color32::from_rgb(255, 130, 130) } else { egui::Color32::from_gray(130) })
                    )
                    .fill(if end_active { egui::Color32::from_rgb(80, 25, 25) } else { egui::Color32::from_gray(38) })
                    .stroke(egui::Stroke::new(1.0, if end_active { egui::Color32::from_rgb(255, 100, 100) } else { egui::Color32::from_gray(60) }))
                    .rounding(3.0)
                    .min_size(egui::vec2(42.0, 18.0));
                    if ui.add(btn_end).on_hover_text("Snap to endpoints / vertices").clicked() {
                        snaps.endpoint = !snaps.endpoint;
                    }
                });
            });
        });
}
