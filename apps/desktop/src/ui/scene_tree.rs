use std::collections::HashSet;
use bevy_egui::{egui, EguiContexts};
use crate::app::AppState;
use crate::commands;
use kpe_schema::joint::JointType;
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

            let mut delete_target: Option<String> = None;
            egui::ScrollArea::vertical().id_salt("scene_tree_scroll").show(ui, |ui| {
                let node = &state.document.recipe.scene;
                tree_node(ui, node, &mut state.document.selection, &mut state.document.multi_selection, &mut state.pending_sketch_edit, &mut delete_target, &mut state.document.hidden_nodes);
            });

            ui.separator();
            ui.label("Joints");
            egui::ScrollArea::vertical().id_salt("joints_scroll").max_height(120.0).show(ui, |ui| {
                let joints = &state.document.recipe.joints;
                if joints.is_empty() {
                    ui.weak("(none)");
                }
                for joint in joints {
                    let is_selected = state.document.joint_selection.as_deref() == Some(&joint.id);
                    let label = format!("{}: {} → {}", joint_type_name(&joint.joint_type), joint.parent_id, joint.child_id);
                    if ui.selectable_label(is_selected, &label).clicked() {
                        state.document.joint_selection = Some(joint.id.clone());
                    }
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                let can_copy = state.document.selection.is_some();
                if ui.add_enabled(can_copy, egui::Button::new("Copy  Ctrl+C")).on_hover_text("Copy selected node to clipboard").clicked() {
                    commands::copy_selected(state);
                }
                if ui.add_enabled(can_copy, egui::Button::new("Cut  Ctrl+X")).on_hover_text("Cut selected node to clipboard").clicked() {
                    commands::cut_selected(state);
                }
                if ui.add_enabled(state.clipboard.is_some(), egui::Button::new("Paste  Ctrl+V")).on_hover_text("Paste node from clipboard").clicked() {
                    commands::paste_clipboard(state);
                }
                if ui.add_enabled(can_copy, egui::Button::new("Dup  Ctrl+D")).on_hover_text("Duplicate selected node").clicked() {
                    commands::duplicate_selected(state);
                }
                if ui.add_enabled(can_copy && state.document.selection.as_deref() != Some("Root"), egui::Button::new("Delete  Del")).on_hover_text("Delete selected node(s)").clicked() {
                    commands::delete_selected_nodes(state);
                }
            });
            ui.horizontal(|ui| {
                let can_copy = state.document.selection.is_some();
                if ui.add_enabled(can_copy, egui::Button::new("Array...")).on_hover_text("Create a linear/rotational array of the selection").clicked() {
                    state.show_array_dialog = true;
                }
                if ui.add_enabled(can_copy, egui::Button::new("Mirror...")).on_hover_text("Mirror the selection across a plane").clicked() {
                    state.show_mirror_dialog = true;
                }
                if ui.add_enabled(can_copy, egui::Button::new("Fillet...")).on_hover_text("Round edges of the selected solid").clicked() {
                    state.show_fillet_dialog = true;
                }
                if ui.add_enabled(can_copy, egui::Button::new("Chamfer...")).on_hover_text("Bevel edges of the selected solid").clicked() {
                    state.show_chamfer_dialog = true;
                }
            });
            ui.horizontal(|ui| {
                if ui.button("+Box").on_hover_text("Add a box primitive").clicked() { commands::add_box(state); }
                if ui.button("+Cyl").on_hover_text("Add a cylinder primitive").clicked() { commands::add_cylinder(state); }
                if ui.button("+Sph").on_hover_text("Add a sphere primitive").clicked() { commands::add_sphere(state); }
                if ui.button("+Sketch").on_hover_text("Add a 2D sketch").clicked() { commands::add_sketch(state); }
            });
            ui.horizontal(|ui| {
                let can_copy = state.document.selection.is_some();
                if ui.add_enabled(can_copy, egui::Button::new("Group")).on_hover_text("Group selection into a compound node").clicked() {
                    let sel = state.document.selection.clone();
                    if let Some(ref id) = sel {
                        commands::add_group(state, id);
                    }
                }
                if ui.add_enabled(can_copy, egui::Button::new("Assembly")).on_hover_text("Wrap selection in an assembly node").clicked() {
                    let sel = state.document.selection.clone();
                    if let Some(ref id) = sel {
                        commands::add_assembly(state, id);
                    }
                }
                if ui.add_enabled(can_copy, egui::Button::new("Joint...")).on_hover_text("Create a joint between two nodes").clicked() {
                    state.show_joint_dialog = true;
                }
            });
        });

    if state.show_array_dialog {
        let params_clone = state.array_params.clone();
        egui::Window::new("Array")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(contexts.ctx_mut(), |ui| {
                ui.horizontal(|ui| { ui.label("Count:"); ui.add(egui::DragValue::new(&mut state.array_params.count).range(1..=100)); });
                ui.separator();
                ui.label("Translation offset:");
                ui.horizontal(|ui| { ui.label("X:"); ui.add(egui::DragValue::new(&mut state.array_params.dx).speed(0.1)); });
                ui.horizontal(|ui| { ui.label("Y:"); ui.add(egui::DragValue::new(&mut state.array_params.dy).speed(0.1)); });
                ui.horizontal(|ui| { ui.label("Z:"); ui.add(egui::DragValue::new(&mut state.array_params.dz).speed(0.1)); });
                ui.separator();
                ui.label("Rotation offset (°):");
                ui.horizontal(|ui| { ui.label("RX:"); ui.add(egui::DragValue::new(&mut state.array_params.rx).speed(1.0)); });
                ui.horizontal(|ui| { ui.label("RY:"); ui.add(egui::DragValue::new(&mut state.array_params.ry).speed(1.0)); });
                ui.horizontal(|ui| { ui.label("RZ:"); ui.add(egui::DragValue::new(&mut state.array_params.rz).speed(1.0)); });
                ui.separator();
                ui.label("Scale multiplier:");
                ui.horizontal(|ui| { ui.label("SX:"); ui.add(egui::DragValue::new(&mut state.array_params.sx).speed(0.01).range(0.01..=100.0)); });
                ui.horizontal(|ui| { ui.label("SY:"); ui.add(egui::DragValue::new(&mut state.array_params.sy).speed(0.01).range(0.01..=100.0)); });
                ui.horizontal(|ui| { ui.label("SZ:"); ui.add(egui::DragValue::new(&mut state.array_params.sz).speed(0.01).range(0.01..=100.0)); });
                ui.separator();
                if ui.button("Create").clicked() {
                    commands::array_selected(state, &params_clone);
                    state.show_array_dialog = false;
                }
                if ui.button("Cancel").clicked() {
                    state.show_array_dialog = false;
                }
            });
    }

    if state.show_mirror_dialog {
        let mut plane = String::from("XY");
        egui::Window::new("Mirror")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(contexts.ctx_mut(), |ui| {
                ui.label("Mirror plane:");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut plane, "XY".to_string(), "XY  (Z→ -Z)");
                });
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut plane, "XZ".to_string(), "XZ  (Y→ -Y)");
                });
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut plane, "YZ".to_string(), "YZ  (X→ -X)");
                });
                ui.separator();
                if ui.button("Create Mirror").clicked() {
                    commands::mirror_selected(state, &plane);
                    state.show_mirror_dialog = false;
                }
                if ui.button("Cancel").clicked() {
                    state.show_mirror_dialog = false;
                }
            });
    }

    if state.show_fillet_dialog {
        let mut radius = 0.5;
        egui::Window::new("Fillet")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(contexts.ctx_mut(), |ui| {
                ui.horizontal(|ui| { ui.label("Radius:"); ui.add(egui::DragValue::new(&mut radius).speed(0.01).range(0.001..=100.0)); });
                ui.separator();
                if ui.button("Apply Fillet").clicked() {
                    commands::add_fillet(state, radius);
                    state.show_fillet_dialog = false;
                }
                if ui.button("Cancel").clicked() {
                    state.show_fillet_dialog = false;
                }
            });
    }

    if state.show_chamfer_dialog {
        let mut distance = 0.5;
        egui::Window::new("Chamfer")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(contexts.ctx_mut(), |ui| {
                ui.horizontal(|ui| { ui.label("Distance:"); ui.add(egui::DragValue::new(&mut distance).speed(0.01).range(0.001..=100.0)); });
                ui.separator();
                if ui.button("Apply Chamfer").clicked() {
                    commands::add_chamfer(state, distance);
                    state.show_chamfer_dialog = false;
                }
                if ui.button("Cancel").clicked() {
                    state.show_chamfer_dialog = false;
                }
            });
    }

    if state.show_joint_dialog {
        let sel = state.document.selection.clone();
        egui::Window::new("Add Joint")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(contexts.ctx_mut(), |ui| {
                ui.label("Select parent and child nodes, then configure:");
                ui.separator();
                let mut jt = state.new_joint_type.clone();
                ui.horizontal(|ui| { ui.label("Type:"); ui.selectable_value(&mut jt, JointType::Revolute, "Revolute"); });
                ui.horizontal(|ui| { ui.selectable_value(&mut jt, JointType::Prismatic, "Prismatic"); });
                ui.horizontal(|ui| { ui.selectable_value(&mut jt, JointType::Fixed, "Fixed"); });
                ui.horizontal(|ui| { ui.selectable_value(&mut jt, JointType::Ball, "Ball"); });
                state.new_joint_type = jt;

                ui.horizontal(|ui| {
                    ui.label("Pivot:");
                    ui.add(egui::DragValue::new(&mut state.new_joint_pivot[0]).speed(0.1));
                    ui.add(egui::DragValue::new(&mut state.new_joint_pivot[1]).speed(0.1));
                    ui.add(egui::DragValue::new(&mut state.new_joint_pivot[2]).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("Axis:");
                    ui.add(egui::DragValue::new(&mut state.new_joint_axis[0]).speed(0.1));
                    ui.add(egui::DragValue::new(&mut state.new_joint_axis[1]).speed(0.1));
                    ui.add(egui::DragValue::new(&mut state.new_joint_axis[2]).speed(0.1));
                });
                ui.separator();
                if ui.button("Create Joint").clicked() {
                    commands::add_joint(state, &sel);
                    state.show_joint_dialog = false;
                }
                if ui.button("Cancel").clicked() {
                    state.show_joint_dialog = false;
                }
            });
    }
}

fn joint_type_name(jt: &JointType) -> &'static str {
    match jt {
        JointType::Revolute => "Revolute",
        JointType::Prismatic => "Prismatic",
        JointType::Fixed => "Fixed",
        JointType::Ball => "Ball",
    }
}

fn tree_node(ui: &mut egui::Ui, node: &GeometryNode, selection: &mut Option<String>, multi_selection: &mut Vec<String>, pending_edit: &mut Option<String>, delete_target: &mut Option<String>, hidden_nodes: &mut HashSet<String>) {
    let is_selected = selection.as_deref() == Some(&node.id) || multi_selection.contains(&node.id);
    let is_hidden = hidden_nodes.contains(&node.id);
    let label = format!("{} ({})", node.id, node_type_name(&node.node_type));

    let response = ui.horizontal(|ui| {
        let eye_label = if is_hidden { "\u{25CB}" } else { "\u{25CF}" };
        let eye_response = ui.selectable_label(false, eye_label);
        if eye_response.clicked() {
            if is_hidden {
                hidden_nodes.remove(&node.id);
            } else {
                hidden_nodes.insert(node.id.clone());
            }
        }
        eye_response.on_hover_text(if is_hidden { "Show node" } else { "Hide node" });

        ui.selectable_label(is_selected, &label)
    }).inner;
    if response.clicked() {
        let ctrl = ui.input(|i| i.modifiers.ctrl);
        if ctrl {
            if multi_selection.contains(&node.id) {
                multi_selection.retain(|id| id != &node.id);
            } else {
                multi_selection.push(node.id.clone());
            }
            if selection.as_deref() == Some(&node.id) {
                *selection = None;
            }
        } else {
            *selection = Some(node.id.clone());
            multi_selection.clear();
        }
    }
    if response.double_clicked() {
        if matches!(node.node_type, GeometryNodeType::Sketch(_)) {
            *pending_edit = Some(node.id.clone());
        }
    }
    if node.id != "Root" {
        response.context_menu(|ui| {
            if ui.button("Delete").clicked() {
                *delete_target = Some(node.id.clone());
                ui.close_menu();
            }
        });
    }
    // Recurse into Fillet/Chamfer children
    if matches!(node.node_type, GeometryNodeType::Fillet(_) | GeometryNodeType::Chamfer(_)) {
        for child in &node.children {
            tree_node(ui, child, selection, multi_selection, pending_edit, delete_target, hidden_nodes);
        }
    }

    if !node.children.is_empty() {
        ui.indent(node.id.clone(), |ui| {
            for child in &node.children {
                tree_node(ui, child, selection, multi_selection, pending_edit, delete_target, hidden_nodes);
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
        GeometryNodeType::Fillet(_) => "Fillet",
        GeometryNodeType::Chamfer(_) => "Chamfer",
        GeometryNodeType::Assembly(_) => "Assembly",
        GeometryNodeType::Compound => "Group",
    }
}
