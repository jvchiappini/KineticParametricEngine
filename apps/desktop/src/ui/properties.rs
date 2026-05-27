use bevy_egui::{egui, EguiContexts};
use crate::app::AppState;
use kpe_schema::geometry::{
    BoxDef, CylinderDef, GeometryNode, GeometryNodeType, SphereDef,
};

pub fn show(contexts: &mut EguiContexts, state: &mut AppState) {
    egui::SidePanel::right("properties_panel")
        .resizable(true)
        .default_width(280.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.add_space(8.0);
            ui.heading("Properties");
            ui.separator();
            ui.add_space(4.0);

            let sel = state.document.selection.clone();
            if let Some(ref sel_id) = sel {
                let node_info = get_node_info(&state.document.recipe.scene, sel_id);
                match node_info {
                    Some(info) => {
                        show_node_properties(ui, &info, state);
                    }
                    None => {
                        ui.label("(node not found)");
                    }
                }
            } else {
                ui.weak("(no selection)");
            }
        });
}

enum NodeInfo {
    Box(BoxDef),
    Cylinder(CylinderDef),
    Sphere(SphereDef),
    Compound,
}

fn get_node_info(node: &GeometryNode, target: &str) -> Option<NodeInfo> {
    if node.id != target {
        for child in &node.children {
            if let found @ Some(_) = get_node_info(child, target) {
                return found;
            }
        }
        return None;
    }
    Some(match &node.node_type {
        GeometryNodeType::Box(b) => NodeInfo::Box(b.clone()),
        GeometryNodeType::Cylinder(c) => NodeInfo::Cylinder(c.clone()),
        GeometryNodeType::Sphere(s) => NodeInfo::Sphere(s.clone()),
        _ => NodeInfo::Compound,
    })
}

fn show_node_properties(ui: &mut egui::Ui, info: &NodeInfo, state: &mut AppState) {
    match info {
        NodeInfo::Box(b) => show_box_properties(ui, b.clone(), state),
        NodeInfo::Cylinder(c) => show_cylinder_properties(ui, c.clone(), state),
        NodeInfo::Sphere(s) => show_sphere_properties(ui, s.clone(), state),
        NodeInfo::Compound => {
            ui.label("Group node");
        }
    }
}

fn show_box_properties(
    ui: &mut egui::Ui,
    mut def: BoxDef,
    state: &mut AppState,
) {
    ui.label("Box");

    let orig = def.clone();
    let mut changed = false;
    changed |= float_drag(ui, "Width", &mut def.width, 0.1..=100.0);
    changed |= float_drag(ui, "Height", &mut def.height, 0.1..=100.0);
    changed |= float_drag(ui, "Depth", &mut def.depth, 0.1..=100.0);

    if changed {
        let node_id = state.document.selection.clone().unwrap_or_default();
        let cmd = crate::commands::SetParameterCommand {
            node_id: node_id.clone(),
            param_name: "height".to_string(),
            old_value: orig.height,
            new_value: def.height,
        };
        state.history.execute(Box::new(cmd), &mut state.document);
        update_node_type(&mut state.document, &node_id, GeometryNodeType::Box(def));
        state.mark_dirty();
    }
}

fn show_cylinder_properties(
    ui: &mut egui::Ui,
    mut def: CylinderDef,
    state: &mut AppState,
) {
    ui.label("Cylinder");

    let orig = def.clone();
    let mut changed = false;
    changed |= float_drag(ui, "Radius", &mut def.radius, 0.1..=100.0);
    changed |= float_drag(ui, "Height", &mut def.height, 0.1..=100.0);

    if changed {
        let node_id = state.document.selection.clone().unwrap_or_default();
        let cmd = crate::commands::SetParameterCommand {
            node_id: node_id.clone(),
            param_name: "height".to_string(),
            old_value: orig.height,
            new_value: def.height,
        };
        state.history.execute(Box::new(cmd), &mut state.document);
        update_node_type(&mut state.document, &node_id, GeometryNodeType::Cylinder(def));
        state.mark_dirty();
    }
}

fn show_sphere_properties(
    ui: &mut egui::Ui,
    mut def: SphereDef,
    state: &mut AppState,
) {
    ui.label("Sphere");

    let orig = def.clone();
    let mut changed = false;
    changed |= float_drag(ui, "Radius", &mut def.radius, 0.1..=100.0);

    if changed {
        let node_id = state.document.selection.clone().unwrap_or_default();
        let cmd = crate::commands::SetParameterCommand {
            node_id: node_id.clone(),
            param_name: "radius".to_string(),
            old_value: orig.radius,
            new_value: def.radius,
        };
        state.history.execute(Box::new(cmd), &mut state.document);
        update_node_type(&mut state.document, &node_id, GeometryNodeType::Sphere(def));
        state.mark_dirty();
    }
}

fn float_drag(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f64,
    range: std::ops::RangeInclusive<f64>,
) -> bool {
    let old = *value;
    let mut display = old as f32;
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(
            egui::DragValue::new(&mut display)
                .speed(0.1)
                .range(*range.start() as f32..=*range.end() as f32),
        );
    });
    *value = display as f64;
    (*value - old).abs() > 1e-9
}

fn update_node_type(doc: &mut crate::document::Document, target: &str, new_type: GeometryNodeType) {
    update_node_type_rec(&mut doc.recipe.scene, target, new_type);
    doc.evaluate_node(target);
}

fn update_node_type_rec(node: &mut GeometryNode, target: &str, new_type: GeometryNodeType) {
    if node.id == target {
        node.node_type = new_type;
        return;
    }
    for child in &mut node.children {
        update_node_type_rec(child, target, new_type.clone());
    }
}
