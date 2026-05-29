use std::time::Instant;
use bevy_egui::{egui, EguiContexts};
use crate::app::AppState;
use kpe_schema::geometry::{GeometryNode, GeometryNodeType};
use crate::commands;

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
            Some(format!("Box: {:.2}×{:.2}×{:.2}", b.width, b.height, b.depth))
        }
        GeometryNodeType::Cylinder(c) => {
            Some(format!("Cyl: r{:.2} × h{:.2}", c.radius, c.height))
        }
        GeometryNodeType::Sphere(s) => {
            Some(format!("Sphere: r{:.2}", s.radius))
        }
        _ => None,
    }
}

pub fn show(contexts: &mut EguiContexts, state: &mut AppState) {
    egui::TopBottomPanel::bottom("status_bar")
        .min_height(24.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                let tri_count = state.document.evaluated.triangle_count();
                ui.label(format!("Tris: {}", tri_count));

                ui.separator();

                if let Some(ref sel) = state.document.selection {
                    if let Some(node) = commands::find_node(&state.document.recipe.scene, sel) {
                        if let Some(meas) = node_measurement(node) {
                            ui.label(meas);
                            ui.separator();
                        }
                    }
                }

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
                    ui.label(format!("FPS: {:.0}", timer.fps));
                    ui.ctx().data_mut(|d| d.insert_temp(tid, timer));
                } else {
                    ui.ctx().data_mut(|d| d.insert_temp(tid, FrameTimer::default()));
                    ui.label("FPS: --");
                }
            });
        });
}
