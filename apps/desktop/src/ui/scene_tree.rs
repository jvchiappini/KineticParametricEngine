use std::collections::HashSet;
use bevy_egui::{egui, EguiContexts};
use crate::app::{AppState, InsertPosition};
use crate::{commands, feature_commands};
use kpe_parametric::MoveNodeCommand;
use kpe_schema::joint::JointType;
use kpe_schema::geometry::{GeometryNode, GeometryNodeType};

const DROP_LINE_HEIGHT: f32 = 2.0;
const DROP_HIGHLIGHT_COLOR: egui::Color32 = egui::Color32::from_rgb(100, 180, 255);

pub fn show_content(ui: &mut egui::Ui, state: &mut AppState) {
    let mut delete_target: Option<String> = None;

            // Drag-and-drop state carried through tree traversal
            let mut drag_ended = false;
            let mut drag_source: Option<String> = None;  // local copy
            let mut local_drag_hover: Option<String> = None;
            let mut local_drag_position = InsertPosition::After;

            // If a drag is already in progress (from previous frame), restore it
            if state.drag_source.is_some() {
                drag_source = state.drag_source.clone();
                // Reset hover each frame — will be recalculated
                state.drag_hover = None;
            }

            egui::ScrollArea::vertical().id_salt("scene_tree_scroll").show(ui, |ui| {
                let node = &state.document.recipe.scene;
                tree_node(
                    ui, node, "",
                    &mut state.document.selection,
                    &mut state.document.multi_selection,
                    &mut state.document.pivot_selection,
                    &mut state.pending_sketch_edit,
                    &mut delete_target,
                    &mut state.document.hidden_nodes,
                    &mut drag_source,
                    &mut local_drag_hover,
                    &mut local_drag_position,
                    &mut drag_ended,
                );
            });

            // ── Drop resolution ──
            if drag_ended {
                if let (Some(ref src), Some(ref hover)) = (drag_source, local_drag_hover) {
                    if src != hover {
                        // Prevent dropping onto a descendant (would create a cycle)
                        let scene = &state.document.recipe.scene;
                        let cannot_drop = is_descendant_of(scene, src, hover)
                            // Before/After Root is invalid — Root has no parent
                            || (hover == "Root" && local_drag_position != InsertPosition::AsChild);
                        if !cannot_drop {
                            let (target_parent_id, insert_index) = compute_drop_params(
                                scene, hover, local_drag_position,
                            );
                            // Only execute if the source would actually move
                            if let (Some(tpid), Some(idx)) = (target_parent_id, insert_index) {
                                if tpid != *src || {
                                    is_valid_index(&state.document.recipe.scene, src, &tpid, idx)
                                } {
                                    let cmd = MoveNodeCommand {
                                        source_id: src.clone(),
                                        target_parent_id: tpid,
                                        insert_index: idx,
                                        old_parent_id: String::new(), // filled on execute
                                        old_index: 0,
                                    };
                                    state.execute(Box::new(cmd));
                                }
                            }
                        }
                    }
                }
                // Clean up drag state
                state.drag_source = None;
                state.drag_hover = None;
            } else {
                // Persist drag state for next frame
                if drag_source.is_some() {
                    state.drag_source = drag_source.clone();
                    state.drag_hover = local_drag_hover;
                    state.drag_position = local_drag_position;
                }
            }

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
                    feature_commands::duplicate_selected(state);
                }
                if ui.add_enabled(can_copy && state.document.selection.as_deref() != Some("Root"), egui::Button::new("Delete  Del")).on_hover_text("Delete selected node(s)").clicked() {
                    feature_commands::delete_selected_nodes(state);
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
                if ui.button("+Box").on_hover_text("Add a box primitive").clicked() { feature_commands::add_box(state); }
                if ui.button("+Cyl").on_hover_text("Add a cylinder primitive").clicked() { feature_commands::add_cylinder(state); }
                if ui.button("+Sph").on_hover_text("Add a sphere primitive").clicked() { feature_commands::add_sphere(state); }
                if ui.button("+Sketch").on_hover_text("Add a 2D sketch").clicked() { feature_commands::add_sketch(state); }
            });
            ui.horizontal(|ui| {
                let can_copy = state.document.selection.is_some();
                if ui.add_enabled(can_copy, egui::Button::new("Group")).on_hover_text("Group selection into a compound node").clicked() {
                    let sel = state.document.selection.clone();
                    if let Some(ref id) = sel {
                        feature_commands::add_group(state, id);
                    }
                }
                if ui.add_enabled(can_copy, egui::Button::new("Assembly")).on_hover_text("Wrap selection in an assembly node").clicked() {
                    let sel = state.document.selection.clone();
                    if let Some(ref id) = sel {
                        feature_commands::add_assembly(state, id);
                    }
                }
                if ui.add_enabled(can_copy, egui::Button::new("Joint...")).on_hover_text("Create a joint between two nodes").clicked() {
                    state.show_joint_dialog = true;
                }
            });
}

pub fn show_dialogs(contexts: &mut EguiContexts, state: &mut AppState) {
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
                    feature_commands::array_selected(state, &params_clone);
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
                    feature_commands::mirror_selected(state, &plane);
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
                    feature_commands::add_fillet(state, radius);
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
                    feature_commands::add_chamfer(state, distance);
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
                ui.horizontal(|ui| {
                    ui.label("Type:");
                    ui.selectable_value(&mut jt, JointType::Revolute { axis: [0.0, 1.0, 0.0] }, "Revolute");
                });
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut jt, JointType::Prismatic { axis: [1.0, 0.0, 0.0] }, "Prismatic");
                });
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut jt, JointType::Fixed, "Fixed");
                });
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut jt, JointType::Ball, "Ball");
                    ui.selectable_value(&mut jt, JointType::Cylindrical { axis: [0.0, 1.0, 0.0] }, "Cylindrical");
                });
                state.new_joint_type = jt;

                ui.label("Parent Frame (position):");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut state.new_joint_parent_frame.position[0]).speed(0.1).prefix("x:"));
                    ui.add(egui::DragValue::new(&mut state.new_joint_parent_frame.position[1]).speed(0.1).prefix("y:"));
                    ui.add(egui::DragValue::new(&mut state.new_joint_parent_frame.position[2]).speed(0.1).prefix("z:"));
                });
                ui.label("Parent Frame (orientation °):");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut state.new_joint_parent_frame.orientation[0]).speed(1.0).prefix("rx:"));
                    ui.add(egui::DragValue::new(&mut state.new_joint_parent_frame.orientation[1]).speed(1.0).prefix("ry:"));
                    ui.add(egui::DragValue::new(&mut state.new_joint_parent_frame.orientation[2]).speed(1.0).prefix("rz:"));
                });
                ui.label("Child Frame (position):");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut state.new_joint_child_frame.position[0]).speed(0.1).prefix("x:"));
                    ui.add(egui::DragValue::new(&mut state.new_joint_child_frame.position[1]).speed(0.1).prefix("y:"));
                    ui.add(egui::DragValue::new(&mut state.new_joint_child_frame.position[2]).speed(0.1).prefix("z:"));
                });
                ui.separator();
                if ui.button("Create Joint").clicked() {
                    feature_commands::add_joint(state, &sel);
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
        JointType::Revolute { .. } => "Revolute",
        JointType::Prismatic { .. } => "Prismatic",
        JointType::Cylindrical { .. } => "Cylindrical",
        JointType::Universal { .. } => "Universal",
        JointType::Ball => "Ball",
        JointType::Planar { .. } => "Planar",
        JointType::Screw { .. } => "Screw",
        JointType::Fixed => "Fixed",
        JointType::SixDOF => "6DOF",
    }
}

/// Render a single tree node and recurse into its children.
///
/// Parameters prefixed `drag_*` are used for drag-and-drop state.
/// `drag_ended` is set to `true` when the drag source's response fires `drag_released()`.
#[allow(clippy::too_many_arguments)]
fn tree_node(
    ui: &mut egui::Ui,
    node: &GeometryNode,
    _parent_id: &str,
    selection: &mut Option<String>,
    multi_selection: &mut Vec<String>,
    _pivot_selection: &mut Option<(String, String)>,
    pending_edit: &mut Option<String>,
    delete_target: &mut Option<String>,
    hidden_nodes: &mut HashSet<String>,
    drag_source: &mut Option<String>,
    drag_hover: &mut Option<String>,
    drag_position: &mut InsertPosition,
    drag_ended: &mut bool,
) {
    let is_selected = selection.as_deref() == Some(&node.id) || multi_selection.contains(&node.id);
    let is_hidden = hidden_nodes.contains(&node.id);
    let is_dragging = drag_source.is_some();
    let is_drag_source = drag_source.as_deref() == Some(&node.id);
    let label = format!("{} ({})", node.id, node_type_name(&node.node_type));

    // ── Row: eye + label ──
    let inner = ui.horizontal(|ui| {
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

        // Label with drag-and-drop support and selection highlight
        let sense = if node.id != "Root" { egui::Sense::click_and_drag() } else { egui::Sense::click() };
        let bg_fill = if is_selected { ui.visuals().selection.bg_fill } else { egui::Color32::TRANSPARENT };
        egui::Frame::none()
            .fill(bg_fill)
            .show(ui, |ui| {
                ui.add(egui::Label::new(&label).sense(sense).selectable(false))
            })
            .inner
    });

    let label_response = inner.inner;
    let rect = label_response.rect;

    // ── Drag initiation ──
    if node.id != "Root" && label_response.drag_started() {
        *drag_source = Some(node.id.clone());
        *drag_hover = None; // reset hover target
    }

    // ── Drag release detection ──
    if is_drag_source && label_response.drag_stopped() {
        *drag_ended = true;
    }

    // ── Drop target detection during drag ──
    // Root can only be AsChild target (can't be Before/After Root)
    if is_dragging && !is_drag_source {
        if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
            if rect.contains(pointer_pos) {
                let pos = if node.id == "Root" {
                    InsertPosition::AsChild
                } else {
                    let local_y = (pointer_pos.y - rect.top()) / rect.height();
                    if local_y < 0.3 {
                        InsertPosition::Before
                    } else if local_y > 0.7 {
                        InsertPosition::After
                    } else {
                        InsertPosition::AsChild
                    }
                };
                *drag_hover = Some(node.id.clone());
                *drag_position = pos;
            }
        }
    }

    // ── Visual feedback: draw drop indicator ──
    if is_dragging && !is_drag_source {
        // Check if this node is the hover target
        if drag_hover.as_deref() == Some(&node.id) {
            let painter = ui.painter();
            match drag_position {
                InsertPosition::Before => {
                    painter.line_segment(
                        [rect.left_top(), rect.right_top()],
                        (DROP_LINE_HEIGHT, DROP_HIGHLIGHT_COLOR),
                    );
                }
                InsertPosition::After => {
                    painter.line_segment(
                        [rect.left_bottom(), rect.right_bottom()],
                        (DROP_LINE_HEIGHT, DROP_HIGHLIGHT_COLOR),
                    );
                }
                InsertPosition::AsChild => {
                    painter.rect_stroke(
                        rect.expand(1.0),
                        egui::Rounding::same(2.0),
                        egui::Stroke::new(2.0, DROP_HIGHLIGHT_COLOR),
                    );
                }
            }
        }
    }

    // ── Selection on click ──
    if label_response.clicked() {
        _pivot_selection.take(); // clear any pivot selection
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

    // ── Double-click to edit sketch ──
    if label_response.double_clicked() {
        if matches!(node.node_type, GeometryNodeType::Sketch(_)) {
            *pending_edit = Some(node.id.clone());
        }
    }

    // ── Context menu ──
    if node.id != "Root" {
        label_response.context_menu(|menu_ui: &mut egui::Ui| {
            if menu_ui.button("Delete").clicked() {
                *delete_target = Some(node.id.clone());
                menu_ui.close_menu();
            }
        });
    }

    // ── Recurse into children (Fillet/Chamfer inline, others indented) ──
    if matches!(node.node_type, GeometryNodeType::Fillet(_) | GeometryNodeType::Chamfer(_)) {
        for child in &node.children {
            tree_node(
                ui, child, &node.id,
                selection, multi_selection, _pivot_selection,
                pending_edit, delete_target, hidden_nodes,
                drag_source, drag_hover, drag_position, drag_ended,
            );
        }
    }

    if !node.children.is_empty() {
        ui.indent(node.id.clone(), |ui| {
            for child in &node.children {
                tree_node(
                    ui, child, &node.id,
                    selection, multi_selection, _pivot_selection,
                    pending_edit, delete_target, hidden_nodes,
                    drag_source, drag_hover, drag_position, drag_ended,
                );
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
        GeometryNodeType::JointGroup => "Joint Group",
    }
}

/// Compute the target parent ID and insert index for a drop operation.
fn compute_drop_params(
    scene: &GeometryNode,
    hover_id: &str,
    position: InsertPosition,
) -> (Option<String>, Option<usize>) {
    match position {
        InsertPosition::Before | InsertPosition::After => {
            // Need parent of the hovered node
            if let Some((parent, idx)) = kpe_parametric::find_parent_and_index(scene, hover_id) {
                let insert_idx = match position {
                    InsertPosition::Before => idx,
                    InsertPosition::After => idx + 1,
                    _ => idx,
                };
                (Some(parent.id.clone()), Some(insert_idx))
            } else {
                (None, None)
            }
        }
        InsertPosition::AsChild => {
            // Target parent is the hovered node itself
            (Some(hover_id.to_string()), Some(0))
        }
    }
}

/// Check if `descendant_id` is a descendant of `ancestor_id` in the scene tree.
fn is_descendant_of(
    node: &GeometryNode,
    ancestor_id: &str,
    descendant_id: &str,
) -> bool {
    if node.id == ancestor_id {
        return find_child_by_id_any_depth(node, descendant_id);
    }
    for child in &node.children {
        if is_descendant_of(child, ancestor_id, descendant_id) {
            return true;
        }
    }
    false
}

/// Recursively search for a node by ID anywhere in the subtree.
fn find_child_by_id_any_depth(node: &GeometryNode, target: &str) -> bool {
    for child in &node.children {
        if child.id == target || find_child_by_id_any_depth(child, target) {
            return true;
        }
    }
    false
}

/// Check if moving `src_id` to `target_parent_id` at `index` would change anything.
fn is_valid_index(scene: &GeometryNode, src_id: &str, target_parent_id: &str, index: usize) -> bool {
    if target_parent_id == src_id {
        return false;
    }
    // If target parent is the current parent, check if index is the same
    if let Some((parent, current_idx)) = kpe_parametric::find_parent_and_index(scene, src_id) {
        if parent.id == target_parent_id {
            // Dropping at same index is a no-op
            // Removing src shifts later indices down by 1
            let effective_idx = if current_idx < index { index - 1 } else { index };
            if effective_idx == current_idx {
                return false;
            }
        }
    }
    true
}
