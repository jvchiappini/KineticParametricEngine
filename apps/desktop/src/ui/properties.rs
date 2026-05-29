use bevy_egui::{egui, EguiContexts};
use bevy_egui::egui::color_picker;
use crate::app::AppState;
use crate::commands::{SetParameterCommand, SetJointValueCommand};
use kpe_schema::geometry::{
    BoxDef, CylinderDef, ExtrudeDef, RevolveDef, GeometryNode, GeometryNodeType, SphereDef, TransformOp,
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

            let joint_id = state.document.joint_selection.clone();
            if let Some(ref jid) = joint_id {
                show_joint_properties(ui, jid, state);
                return;
            }

            let sel = state.document.selection.clone();
            if let Some(ref sel_id) = sel {
                let node_clone = find_node(&state.document.recipe.scene, sel_id).cloned();
                match node_clone {
                    Some(n) => {
                        show_all_properties(ui, &n, sel_id, state);
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

fn find_node<'a>(node: &'a GeometryNode, target: &str) -> Option<&'a GeometryNode> {
    if node.id == target { return Some(node); }
    for child in &node.children {
        if let found @ Some(_) = find_node(child, target) { return found; }
    }
    None
}

fn find_node_mut<'a>(node: &'a mut GeometryNode, target: &str) -> Option<&'a mut GeometryNode> {
    if node.id == target { return Some(node); }
    for child in &mut node.children {
        if let found @ Some(_) = find_node_mut(child, target) { return found; }
    }
    None
}

fn show_all_properties(ui: &mut egui::Ui, node: &GeometryNode, node_id: &str, state: &mut AppState) {
    // Node type header
    ui.label(format!("Type: {}", node_type_name(&node.node_type)));
    ui.separator();

    // Geometry-specific properties
    match &node.node_type {
        GeometryNodeType::Box(b) => show_box_properties(ui, b.clone(), node_id, state),
        GeometryNodeType::Cylinder(c) => show_cylinder_properties(ui, c.clone(), node_id, state),
        GeometryNodeType::Sphere(s) => show_sphere_properties(ui, s.clone(), node_id, state),
        GeometryNodeType::Extrude(e) => show_extrude_properties(ui, e.clone(), node_id, state),
        GeometryNodeType::Revolve(r) => show_revolve_properties(ui, r.clone(), node_id, state),
        GeometryNodeType::Fillet(f) => { ui.label(format!("Radius: {:.2}", f.radius)); }
        GeometryNodeType::Chamfer(c) => { ui.label(format!("Distance: {:.2}", c.distance)); }
        GeometryNodeType::Assembly(_) => { ui.label("Container for grouped parts"); }
        _ => {}
    }

    ui.separator();

    // Transform
    show_transform_properties(ui, node_id, state);

    ui.separator();

    // Color
    show_color_picker(ui, node_id, state);
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

fn show_transform_properties(ui: &mut egui::Ui, node_id: &str, state: &mut AppState) {
    ui.label("Transform");

    let node = find_node_mut(&mut state.document.recipe.scene, node_id);
    let node = match node { Some(n) => n, None => return };

    let tf = node.transform.get_or_insert_with(|| TransformOp {
        translation: None, rotation: None, scale: None,
    });

    let mut tx = tf.translation.unwrap_or([0.0; 3]);
    let mut rot = tf.rotation.unwrap_or([0.0; 3]);
    let mut sc = tf.scale.unwrap_or([1.0; 3]);

    let mut changed = false;

    ui.horizontal(|ui| { ui.label("Pos:"); changed |= drag3(ui, &mut tx); });
    ui.horizontal(|ui| { ui.label("Rot:"); changed |= drag3_rot(ui, &mut rot); });
    ui.horizontal(|ui| { ui.label("Scl:"); changed |= drag3_scale(ui, &mut sc); });

    if changed {
        tf.translation = Some(tx);
        tf.rotation = Some(rot);
        tf.scale = Some(sc);
        state.document.evaluate_node(node_id);
        state.mark_dirty();
    }
}

fn drag3(ui: &mut egui::Ui, v: &mut [f64; 3]) -> bool {
    let mut changed = false;
    changed |= ui.add(egui::DragValue::new(&mut v[0]).speed(0.1).prefix("X ")).changed();
    changed |= ui.add(egui::DragValue::new(&mut v[1]).speed(0.1).prefix("Y ")).changed();
    changed |= ui.add(egui::DragValue::new(&mut v[2]).speed(0.1).prefix("Z ")).changed();
    changed
}

fn drag3_rot(ui: &mut egui::Ui, v: &mut [f64; 3]) -> bool {
    let mut changed = false;
    changed |= ui.add(egui::DragValue::new(&mut v[0]).speed(1.0).prefix("X ")).changed();
    changed |= ui.add(egui::DragValue::new(&mut v[1]).speed(1.0).prefix("Y ")).changed();
    changed |= ui.add(egui::DragValue::new(&mut v[2]).speed(1.0).prefix("Z ")).changed();
    changed
}

fn drag3_scale(ui: &mut egui::Ui, v: &mut [f64; 3]) -> bool {
    let mut changed = false;
    changed |= ui.add(egui::DragValue::new(&mut v[0]).speed(0.01).range(0.01..=100.0).prefix("X ")).changed();
    changed |= ui.add(egui::DragValue::new(&mut v[1]).speed(0.01).range(0.01..=100.0).prefix("Y ")).changed();
    changed |= ui.add(egui::DragValue::new(&mut v[2]).speed(0.01).range(0.01..=100.0).prefix("Z ")).changed();
    changed
}

// ── Primitive properties ───────────────────────

fn show_box_properties(ui: &mut egui::Ui, mut def: BoxDef, node_id: &str, state: &mut AppState) {
    let (w_changed, w_old, w_new) = float_drag(ui, "Width", &mut def.width, 0.1..=100.0);
    let (h_changed, h_old, h_new) = float_drag(ui, "Height", &mut def.height, 0.1..=100.0);
    let (d_changed, d_old, d_new) = float_drag(ui, "Depth", &mut def.depth, 0.1..=100.0);

    if w_changed || h_changed || d_changed {
        if w_changed { exec_param(state, node_id, "width", w_old, w_new); }
        if h_changed { exec_param(state, node_id, "height", h_old, h_new); }
        if d_changed { exec_param(state, node_id, "depth", d_old, d_new); }
        update_node_type(state, node_id, GeometryNodeType::Box(def));
    }
}

fn show_cylinder_properties(ui: &mut egui::Ui, mut def: CylinderDef, node_id: &str, state: &mut AppState) {
    let (r_changed, r_old, r_new) = float_drag(ui, "Radius", &mut def.radius, 0.1..=100.0);
    let (h_changed, h_old, h_new) = float_drag(ui, "Height", &mut def.height, 0.1..=100.0);

    if r_changed || h_changed {
        if r_changed { exec_param(state, node_id, "radius", r_old, r_new); }
        if h_changed { exec_param(state, node_id, "height", h_old, h_new); }
        update_node_type(state, node_id, GeometryNodeType::Cylinder(def));
    }
}

fn show_sphere_properties(ui: &mut egui::Ui, mut def: SphereDef, node_id: &str, state: &mut AppState) {
    let (r_changed, r_old, r_new) = float_drag(ui, "Radius", &mut def.radius, 0.1..=100.0);
    if r_changed {
        exec_param(state, node_id, "radius", r_old, r_new);
        update_node_type(state, node_id, GeometryNodeType::Sphere(def));
    }
}

fn show_extrude_properties(ui: &mut egui::Ui, mut def: ExtrudeDef, node_id: &str, state: &mut AppState) {
    ui.label("Extrude");
    let mut taper = def.taper_angle.unwrap_or(0.0);
    let (d_changed, d_old, d_new) = float_drag(ui, "Distance", &mut def.distance, 0.1..=1000.0);
    let (t_changed, t_old, t_new) = float_drag(ui, "Taper °", &mut taper, -60.0..=60.0);

    if d_changed || t_changed {
        if d_changed { exec_param(state, node_id, "distance", d_old, d_new); }
        if t_changed { exec_param(state, node_id, "taper_angle", t_old, t_new); }
        def.taper_angle = if taper == 0.0 { None } else { Some(taper) };
        update_node_type(state, node_id, GeometryNodeType::Extrude(def));
    }
}

fn show_revolve_properties(ui: &mut egui::Ui, mut def: RevolveDef, node_id: &str, state: &mut AppState) {
    ui.label("Revolve");
    let (a_changed, a_old, a_new) = float_drag(ui, "Angle", &mut def.angle, 0.1..=360.0);
    if a_changed {
        exec_param(state, node_id, "angle", a_old, a_new);
        update_node_type(state, node_id, GeometryNodeType::Revolve(def));
    }
}

// ── Color picker ───────────────────────────────

fn show_color_picker(ui: &mut egui::Ui, node_id: &str, state: &mut AppState) {
    let current_color = get_node_color(&state.document.recipe.scene, node_id);
    let mut rgb = hex_to_srgb(&current_color);

    ui.label("Color");
    if color_picker::color_edit_button_srgb(ui, &mut rgb).changed() {
        let hex = format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
        set_node_color(&mut state.document.recipe.scene, node_id, Some(hex));
        state.document.evaluate_node(node_id);
        state.mark_dirty();
    }
    if ui.button("Reset").clicked() {
        set_node_color(&mut state.document.recipe.scene, node_id, None);
        state.document.evaluate_node(node_id);
        state.mark_dirty();
    }
}

fn hex_to_srgb(hex: &Option<String>) -> [u8; 3] {
    match hex {
        Some(h) if h.len() >= 6 => {
            let h = h.trim_start_matches('#');
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&h[0..2], 16),
                u8::from_str_radix(&h[2..4], 16),
                u8::from_str_radix(&h[4..6], 16),
            ) {
                return [r, g, b];
            }
            [128, 128, 128]
        }
        _ => [128, 128, 128],
    }
}

fn get_node_color(node: &GeometryNode, target: &str) -> Option<String> {
    if node.id == target { return node.color.clone(); }
    for child in &node.children {
        if let found @ Some(_) = get_node_color(child, target) {
            return found;
        }
    }
    None
}

fn set_node_color(node: &mut GeometryNode, target: &str, color: Option<String>) {
    if node.id == target {
        node.color = color;
        return;
    }
    for child in &mut node.children {
        set_node_color(child, target, color.clone());
    }
}

// ── Helpers ────────────────────────────────────

fn exec_param(state: &mut AppState, node_id: &str, name: &str, old: f64, new: f64) {
    let cmd = SetParameterCommand {
        node_id: node_id.to_string(),
        param_name: name.to_string(),
        old_value: old,
        new_value: new,
    };
    state.history.execute(Box::new(cmd), &mut state.document);
}

fn update_node_type(state: &mut AppState, target: &str, new_type: GeometryNodeType) {
    update_node_type_rec(&mut state.document.recipe.scene, target, new_type);
    state.document.evaluate_node(target);
    state.mark_dirty();
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

fn float_drag(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f64,
    range: std::ops::RangeInclusive<f64>,
) -> (bool, f64, f64) {
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
    let changed = (*value - old).abs() > 1e-9;
    (changed, old, *value)
}

// ── Joint properties ────────────────────────────

fn show_joint_properties(ui: &mut egui::Ui, joint_id: &str, state: &mut AppState) {
    let joint = state.document.recipe.joints.iter()
        .find(|j| j.id == joint_id)
        .cloned();
    let joint = match joint {
        Some(j) => j,
        None => { ui.label("(joint not found)"); return; }
    };

    ui.label(format!("Joint: {} ({})", joint.id, match joint.joint_type {
        kpe_schema::joint::JointType::Revolute => "Revolute",
        kpe_schema::joint::JointType::Prismatic => "Prismatic",
        kpe_schema::joint::JointType::Fixed => "Fixed",
        kpe_schema::joint::JointType::Ball => "Ball",
    }));
    ui.label(format!("Parent: {}", joint.parent_id));
    ui.label(format!("Child: {}", joint.child_id));
    ui.separator();

    let mut val = joint.current_value as f32;
    let label = match joint.joint_type {
        kpe_schema::joint::JointType::Revolute | kpe_schema::joint::JointType::Ball => "Angle (°)",
        kpe_schema::joint::JointType::Prismatic => "Distance",
        kpe_schema::joint::JointType::Fixed => "Value",
    };
    ui.horizontal(|ui| {
        ui.label(label);
        if ui.add(egui::Slider::new(&mut val, -180.0..=180.0)).changed() {
            let new_val = val as f64;
            let cmd = SetJointValueCommand {
                joint_id: joint.id.clone(),
                old_value: joint.current_value,
                new_value: new_val,
            };
            state.history.execute(Box::new(cmd), &mut state.document);
            state.mark_dirty();
        }
    });

    ui.separator();
    if ui.button("Clear joint selection").clicked() {
        state.document.joint_selection = None;
    }
}