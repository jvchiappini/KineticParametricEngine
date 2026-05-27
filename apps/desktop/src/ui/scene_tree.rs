use bevy_egui::{egui, EguiContexts};
use crate::app::AppState;
use kpe_schema::geometry::{GeometryNode, GeometryNodeType};

pub fn show(contexts: &mut EguiContexts, state: &mut AppState) {
    egui::SidePanel::left("scene_tree")
        .resizable(true)
        .default_width(220.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.add_space(8.0);
            ui.heading("Scene");
            ui.separator();
            ui.add_space(4.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                let node = &state.document.recipe.scene;
                tree_node(ui, node, &mut state.document.selection);
            });
        });
}

fn tree_node(ui: &mut egui::Ui, node: &GeometryNode, selection: &mut Option<String>) {
    let is_selected = selection.as_deref() == Some(&node.id);
    let label = format!("{} ({})", node.id, node_type_name(&node.node_type));

    let response = ui.selectable_label(is_selected, &label);
    if response.clicked() {
        *selection = Some(node.id.clone());
    }

    if !node.children.is_empty() {
        ui.indent(node.id.clone(), |ui| {
            for child in &node.children {
                tree_node(ui, child, selection);
            }
        });
    }
}

fn node_type_name(nt: &GeometryNodeType) -> &'static str {
    match nt {
        GeometryNodeType::Box(_) => "Box",
        GeometryNodeType::Cylinder(_) => "Cylinder",
        GeometryNodeType::Sphere(_) => "Sphere",
        GeometryNodeType::Mesh(_) => "Mesh",
        GeometryNodeType::Sketch(_) => "Sketch",
        GeometryNodeType::Extrude(_) => "Extrude",
        GeometryNodeType::Revolve(_) => "Revolve",
        GeometryNodeType::Sweep(_) => "Sweep",
        GeometryNodeType::Compound => "Group",
    }
}
