use bevy_egui::{egui, EguiContexts};
use bevy_egui::egui::color_picker;
use crate::app::AppState;
use kpe_parametric::commands::{
    SetParameterCommand, SetJointValueCommand, SetSketchCommand,
    find_node, find_node_mut,
};
use kpe_parametric::commands::pivot::{
    AddPivotCommand, RemovePivotCommand, SetPivotTransformCommand, PivotField,
};
use kpe_schema::geometry::{
    BoxDef, CylinderDef, ExtrudeDef, PivotSpace, PivotTransform, RevolveDef,
    GeometryNode, GeometryNodeType, SketchDef, SphereDef, TransformOp,
};

pub fn show_entity_info_content(ui: &mut egui::Ui, state: &mut AppState) {
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
        GeometryNodeType::Sketch(s) => show_sketch_properties(ui, s.clone(), node_id, state),
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
        GeometryNodeType::JointGroup => "Joint Group",
    }
}

fn show_transform_properties(ui: &mut egui::Ui, node_id: &str, state: &mut AppState) {
    ui.label("Transform");

    // ── Base transform (read + release borrow before mutating state) ──
    let (mut tx, mut rot, mut sc) = {
        let node = match find_node_mut(&mut state.document.recipe.scene, node_id) {
            Some(n) => n,
            None => return,
        };
        let tf = node.transform.get_or_insert_with(|| TransformOp {
            translation: None, rotation: None, scale: None, pivots: Vec::new(),
        });
        (
            tf.translation.unwrap_or([0.0; 3]),
            tf.rotation.unwrap_or([0.0; 3]),
            tf.scale.unwrap_or([1.0; 3]),
        )
    };

    let mut changed = false;
    ui.horizontal(|ui| { ui.label("Pos:"); changed |= drag3(ui, &mut tx); });
    ui.horizontal(|ui| { ui.label("Rot:"); changed |= drag3_rot(ui, &mut rot); });
    ui.horizontal(|ui| { ui.label("Scl:"); changed |= drag3_scale(ui, &mut sc); });

    if changed {
        let node = find_node_mut(&mut state.document.recipe.scene, node_id).unwrap();
        let tf = node.transform.get_or_insert_with(|| TransformOp {
            translation: None, rotation: None, scale: None, pivots: Vec::new(),
        });
        tf.translation = Some(tx);
        tf.rotation = Some(rot);
        tf.scale = Some(sc);
        state.document.evaluate_node(node_id);
        state.mark_dirty();
    }

    // ── Pivot tree ────────────────────────────────────────────────
    ui.separator();
    ui.label("Pivots");

    // Add pivot button
    if ui.button("+ Add Pivot").clicked() {
        let pivots_len = {
            let node = find_node_mut(&mut state.document.recipe.scene, node_id).unwrap();
            let tf = node.transform.get_or_insert_with(|| TransformOp {
                translation: None, rotation: None, scale: None, pivots: Vec::new(),
            });
            tf.pivots.len()
        };
        let pivot = PivotTransform {
            id: format!("{}/pivot_{}", node_id, pivots_len + 1),
            pivot: [0.0; 3],
            space: PivotSpace::Local,
            translation: None,
            rotation: None,
            scale: None,
        };
        state.execute(Box::new(AddPivotCommand {
            node_id: node_id.to_string(),
            pivot,
        }));
    }

    // Clone the pivot list for render to avoid borrow conflicts
    let pivots: Vec<PivotTransform> = {
        let node = match find_node_mut(&mut state.document.recipe.scene, node_id) {
            Some(n) => n,
            None => return,
        };
        let tf = node.transform.get_or_insert_with(|| TransformOp {
            translation: None, rotation: None, scale: None, pivots: Vec::new(),
        });
        tf.pivots.clone()
    };

    if pivots.is_empty() {
        ui.weak("(none — transform at center)");
    } else {
        let mut delete_id: Option<String> = None;
        ui.indent("pivot_tree", |ui| {
            for (i, pv) in pivots.iter().enumerate() {
                let pivot_id = pv.id.clone();
                let is_selected = state.document.pivot_selection.as_ref()
                    .map(|(n, p)| n == node_id && p == &pv.id)
                    .unwrap_or(false);

                egui::CollapsingHeader::new(format!("Pivot {}: {}", i + 1, &pv.id))
                    .id_salt(format!("pivot_{}_{}", node_id, &pv.id))
                    .default_open(is_selected)
                    .show(ui, |ui| {
                        // Space toggle: Local / World
                        let old_space = pv.space;
                        let mut space = old_space;
                        ui.horizontal(|ui| {
                            ui.label("Space:");
                            ui.selectable_value(&mut space, PivotSpace::Local, "Local");
                            ui.selectable_value(&mut space, PivotSpace::World, "World");
                        });
                        if space != old_space {
                            if let Some(n) = find_node_mut(&mut state.document.recipe.scene, node_id) {
                                if let Some(tf) = &mut n.transform {
                                    if let Some(p) = tf.pivots.iter_mut().find(|p| p.id == pivot_id) {
                                        p.space = space;
                                    }
                                }
                            }
                            state.document.evaluate_node(node_id);
                            state.mark_dirty();
                        }

                        // Pivot point
                        let mut pp = pv.pivot;
                        if edit_pivot_field(ui, &format!("Pvt"), &mut pp, 0.1, false) {
                            for (j, coord) in pp.iter().enumerate() {
                                let field = match j { 0 => PivotField::PivotX, 1 => PivotField::PivotY, _ => PivotField::PivotZ };
                                let cmd = SetPivotTransformCommand {
                                    node_id: node_id.to_string(),
                                    pivot_id: pivot_id.clone(),
                                    field,
                                    old_value: pv.pivot[j],
                                    new_value: *coord,
                                };
                                state.execute(Box::new(cmd));
                            }
                        }

                        // Translation
                        let mut pv_tx = pv.translation.unwrap_or([0.0; 3]);
                        if edit_pivot_field(ui, "Pos", &mut pv_tx, 0.1, false) {
                            for (j, coord) in pv_tx.iter().enumerate() {
                                let field = match j { 0 => PivotField::TranslationX, 1 => PivotField::TranslationY, _ => PivotField::TranslationZ };
                                let cmd = SetPivotTransformCommand {
                                    node_id: node_id.to_string(),
                                    pivot_id: pivot_id.clone(),
                                    field,
                                    old_value: pv.translation.unwrap_or([0.0; 3])[j],
                                    new_value: *coord,
                                };
                                state.execute(Box::new(cmd));
                            }
                        }

                        // Rotation
                        let mut pv_rot = pv.rotation.unwrap_or([0.0; 3]);
                        if edit_pivot_field(ui, "Rot", &mut pv_rot, 1.0, false) {
                            for (j, coord) in pv_rot.iter().enumerate() {
                                let field = match j { 0 => PivotField::RotationX, 1 => PivotField::RotationY, _ => PivotField::RotationZ };
                                let cmd = SetPivotTransformCommand {
                                    node_id: node_id.to_string(),
                                    pivot_id: pivot_id.clone(),
                                    field,
                                    old_value: pv.rotation.unwrap_or([0.0; 3])[j],
                                    new_value: *coord,
                                };
                                state.execute(Box::new(cmd));
                            }
                        }

                        // Scale
                        let mut pv_sc = pv.scale.unwrap_or([1.0; 3]);
                        if edit_pivot_field(ui, "Scl", &mut pv_sc, 0.01, true) {
                            for (j, coord) in pv_sc.iter().enumerate() {
                                let field = match j { 0 => PivotField::ScaleX, 1 => PivotField::ScaleY, _ => PivotField::ScaleZ };
                                let cmd = SetPivotTransformCommand {
                                    node_id: node_id.to_string(),
                                    pivot_id: pivot_id.clone(),
                                    field,
                                    old_value: pv.scale.unwrap_or([1.0; 3])[j],
                                    new_value: *coord,
                                };
                                state.execute(Box::new(cmd));
                            }
                        }
                    });

                if ui.button(format!("✕")).clicked() {
                    delete_id = Some(pivot_id.clone());
                }
            }
        });

        if let Some(id) = delete_id {
            state.execute(Box::new(RemovePivotCommand {
                node_id: node_id.to_string(),
                pivot_id: id.clone(),
                old_pivot: None,
            }));
            state.document.pivot_selection = None;
        }
    }
}

/// Edit a 3-element drag field for a pivot transform.
fn edit_pivot_field(ui: &mut egui::Ui, label: &str, v: &mut [f64; 3], speed: f64, is_scale: bool) -> bool {
    let mut changed = false;
    let suffix = if label.contains('°') || label.contains("Rot") || label.contains("Angle") { " °" } else if is_scale { "" } else { " mm" };
    ui.horizontal(|ui| {
        ui.label(label);
        let range = if is_scale { 0.01..=100.0 } else { -1e6..=1e6 };
        changed |= ui.add(egui::DragValue::new(&mut v[0]).speed(speed).range(range.clone()).prefix("X ").suffix(suffix)).changed();
        changed |= ui.add(egui::DragValue::new(&mut v[1]).speed(speed).range(range.clone()).prefix("Y ").suffix(suffix)).changed();
        changed |= ui.add(egui::DragValue::new(&mut v[2]).speed(speed).range(range).prefix("Z ").suffix(suffix)).changed();
    });
    changed
}

fn drag3(ui: &mut egui::Ui, v: &mut [f64; 3]) -> bool {
    let mut changed = false;
    changed |= ui.add(egui::DragValue::new(&mut v[0]).speed(1.0).prefix("X ").suffix(" mm")).changed();
    changed |= ui.add(egui::DragValue::new(&mut v[1]).speed(1.0).prefix("Y ").suffix(" mm")).changed();
    changed |= ui.add(egui::DragValue::new(&mut v[2]).speed(1.0).prefix("Z ").suffix(" mm")).changed();
    changed
}

fn drag3_rot(ui: &mut egui::Ui, v: &mut [f64; 3]) -> bool {
    let mut changed = false;
    changed |= ui.add(egui::DragValue::new(&mut v[0]).speed(1.0).prefix("X ").suffix(" °")).changed();
    changed |= ui.add(egui::DragValue::new(&mut v[1]).speed(1.0).prefix("Y ").suffix(" °")).changed();
    changed |= ui.add(egui::DragValue::new(&mut v[2]).speed(1.0).prefix("Z ").suffix(" °")).changed();
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
    let (w_changed, w_old, w_new) = float_drag(ui, "Width", &mut def.width, 0.1..=10000.0);
    let (h_changed, h_old, h_new) = float_drag(ui, "Height", &mut def.height, 0.1..=10000.0);
    let (d_changed, d_old, d_new) = float_drag(ui, "Depth", &mut def.depth, 0.1..=10000.0);

    if w_changed || h_changed || d_changed {
        if w_changed { exec_param(state, node_id, "width", w_old, w_new); }
        if h_changed { exec_param(state, node_id, "height", h_old, h_new); }
        if d_changed { exec_param(state, node_id, "depth", d_old, d_new); }
        update_node_type(state, node_id, GeometryNodeType::Box(def));
    }
}

fn show_cylinder_properties(ui: &mut egui::Ui, mut def: CylinderDef, node_id: &str, state: &mut AppState) {
    let (r_changed, r_old, r_new) = float_drag(ui, "Radius", &mut def.radius, 0.1..=10000.0);
    let (h_changed, h_old, h_new) = float_drag(ui, "Height", &mut def.height, 0.1..=10000.0);

    // Segments drag — early return to avoid ownership conflict with def
    let mut seg = def.segments as u64;
    let seg_changed = ui
        .horizontal(|ui| {
            ui.label("Segments");
            ui.add(egui::DragValue::new(&mut seg).speed(1).range(3..=256)).changed()
        })
        .inner;
    if seg_changed {
        let old_seg = def.segments as f64;
        def.segments = seg as u32;
        exec_param(state, node_id, "segments", old_seg, def.segments as f64);
        update_node_type(state, node_id, GeometryNodeType::Cylinder(def));
        return;
    }

    if r_changed || h_changed {
        if r_changed { exec_param(state, node_id, "radius", r_old, r_new); }
        if h_changed { exec_param(state, node_id, "height", h_old, h_new); }
        update_node_type(state, node_id, GeometryNodeType::Cylinder(def));
    }
}

fn show_sphere_properties(ui: &mut egui::Ui, mut def: SphereDef, node_id: &str, state: &mut AppState) {
    let (r_changed, r_old, r_new) = float_drag(ui, "Radius", &mut def.radius, 0.1..=10000.0);

    // Segments drag — early return to avoid ownership conflict with def
    let mut seg = def.segments as u64;
    let seg_changed = ui
        .horizontal(|ui| {
            ui.label("Segments");
            ui.add(egui::DragValue::new(&mut seg).speed(1).range(6..=256)).changed()
        })
        .inner;
    if seg_changed {
        let old_seg = def.segments as f64;
        def.segments = seg as u32;
        exec_param(state, node_id, "segments", old_seg, def.segments as f64);
        update_node_type(state, node_id, GeometryNodeType::Sphere(def));
        return;
    }

    if r_changed {
        exec_param(state, node_id, "radius", r_old, r_new);
        update_node_type(state, node_id, GeometryNodeType::Sphere(def));
    }
}

fn show_sketch_properties(ui: &mut egui::Ui, def: SketchDef, node_id: &str, state: &mut AppState) {
    ui.label(format!("Plane: {:?}", def.plane));
    ui.label(format!("Primitives: {}", def.primitives.len()));
    ui.separator();
    ui.label("Extrude");
    let mut has_extrude = def.extrude.is_some();
    if ui.checkbox(&mut has_extrude, "Enabled").changed() {
        let mut new_def = def.clone();
        if has_extrude {
            new_def.extrude = Some(ExtrudeDef {
                sketch_id: node_id.to_string(),
                distance: 2.0,
                direction: None,
                cap: true,
                taper_angle: None,
            });
        } else {
            new_def.extrude = None;
        }
        update_sketch_extrude(state, node_id, new_def);
    }
    if let Some(ref ext) = def.extrude {
        let mut dist = ext.distance;
        let mut taper = ext.taper_angle.unwrap_or(0.0);
        let d_changed = float_drag_mut(ui, "Distance", &mut dist, 0.1..=1000.0);
        let t_changed = float_drag_mut(ui, "Taper °", &mut taper, -60.0..=60.0);
        if d_changed || t_changed {
            let mut new_def = def.clone();
            if let Some(ref mut e) = new_def.extrude {
                e.distance = dist;
                e.taper_angle = if taper == 0.0 { None } else { Some(taper) };
            }
            update_sketch_extrude(state, node_id, new_def);
        }
    }
}

fn update_sketch_extrude(state: &mut AppState, node_id: &str, new_def: SketchDef) {
    state.execute(Box::new(SetSketchCommand {
        node_id: node_id.to_string(),
        old_sketch: None,
        new_sketch: new_def,
    }));
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
    state.execute(Box::new(SetParameterCommand {
        node_id: node_id.to_string(),
        param_name: name.to_string(),
        old_value: old,
        new_value: new,
    }));
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

fn float_drag_mut(ui: &mut egui::Ui, label: &str, value: &mut f64, range: std::ops::RangeInclusive<f64>) -> bool {
    let old = *value;
    let mut display = old as f32;
    let suffix = if label.contains('°') || label.contains("Rot") || label.contains("Angle") { " °" } else { " mm" };
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(
            egui::DragValue::new(&mut display)
                .speed(0.1)
                .range(*range.start() as f32..=*range.end() as f32)
                .suffix(suffix),
        );
    });
    *value = display as f64;
    (*value - old).abs() > 1e-9
}

fn float_drag(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f64,
    range: std::ops::RangeInclusive<f64>,
) -> (bool, f64, f64) {
    let old = *value;
    let mut display = old as f32;
    let suffix = if label.contains('°') || label.contains("Rot") || label.contains("Angle") { " °" } else { " mm" };
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(
            egui::DragValue::new(&mut display)
                .speed(0.1)
                .range(*range.start() as f32..=*range.end() as f32)
                .suffix(suffix),
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
        kpe_schema::joint::JointType::Revolute { .. } => "Revolute",
        kpe_schema::joint::JointType::Prismatic { .. } => "Prismatic",
        kpe_schema::joint::JointType::Cylindrical { .. } => "Cylindrical",
        kpe_schema::joint::JointType::Universal { .. } => "Universal",
        kpe_schema::joint::JointType::Ball => "Ball",
        kpe_schema::joint::JointType::Planar { .. } => "Planar",
        kpe_schema::joint::JointType::Screw { .. } => "Screw",
        kpe_schema::joint::JointType::Fixed => "Fixed",
        kpe_schema::joint::JointType::SixDOF => "6DOF",
    }));
    ui.label(format!("Parent: {}", joint.parent_id));
    ui.label(format!("Child: {}", joint.child_id));
    ui.separator();

    // Primary DOF slider
    let dof_label = match joint.joint_type {
        kpe_schema::joint::JointType::Revolute { .. }
        | kpe_schema::joint::JointType::Ball
        | kpe_schema::joint::JointType::Universal { .. }
        | kpe_schema::joint::JointType::Cylindrical { .. } => "Angle (°)",
        kpe_schema::joint::JointType::Prismatic { .. } => "Distance",
        kpe_schema::joint::JointType::Screw { .. } => "Rotation (°)",
        kpe_schema::joint::JointType::Fixed => "—",
        kpe_schema::joint::JointType::Planar { .. } => "X Offset",
        kpe_schema::joint::JointType::SixDOF => "X",
    };
    if joint.current_values.len() >= 1 {
        let mut val = joint.current_values[0] as f32;
        let limits = joint.limits.as_ref().map(|l| (l.primary.min as f32, l.primary.max as f32));
        let (lo, hi) = limits.unwrap_or((-180.0, 180.0));
        ui.horizontal(|ui| {
            ui.label(dof_label);
            if ui.add(egui::Slider::new(&mut val, lo..=hi)).changed() {
                let mut new_values = joint.current_values.clone();
                new_values[0] = val as f64;
                state.execute(Box::new(SetJointValueCommand {
                    joint_id: joint.id.clone(),
                    old_values: joint.current_values.clone(),
                    new_values,
                }));
            }
        });
    }

    // Secondary DOF slider for 2-DOF and 3-DOF joints
    if joint.current_values.len() >= 2 {
        let label2 = match joint.joint_type {
            kpe_schema::joint::JointType::Cylindrical { .. } => "Distance",
            kpe_schema::joint::JointType::Universal { .. } => "Angle 2 (°)",
            kpe_schema::joint::JointType::Planar { .. } => "Y Offset",
            kpe_schema::joint::JointType::SixDOF => "Y",
            _ => "DOF 2",
        };
        let mut val2 = joint.current_values[1] as f32;
        ui.horizontal(|ui| {
            ui.label(label2);
            if ui.add(egui::Slider::new(&mut val2, -180.0..=180.0)).changed() {
                let mut new_values = joint.current_values.clone();
                new_values[1] = val2 as f64;
                state.execute(Box::new(SetJointValueCommand {
                    joint_id: joint.id.clone(),
                    old_values: joint.current_values.clone(),
                    new_values,
                }));
            }
        });
    }

    // Tertiary DOF slider for 3-DOF joints
    if joint.current_values.len() >= 3 {
        let label3 = match joint.joint_type {
            kpe_schema::joint::JointType::Ball => "Yaw (°)",
            kpe_schema::joint::JointType::Planar { .. } => "Rotation (°)",
            kpe_schema::joint::JointType::SixDOF => "Z",
            _ => "DOF 3",
        };
        let mut val3 = joint.current_values[2] as f32;
        ui.horizontal(|ui| {
            ui.label(label3);
            if ui.add(egui::Slider::new(&mut val3, -180.0..=180.0)).changed() {
                let mut new_values = joint.current_values.clone();
                new_values[2] = val3 as f64;
                state.execute(Box::new(SetJointValueCommand {
                    joint_id: joint.id.clone(),
                    old_values: joint.current_values.clone(),
                    new_values,
                }));
            }
        });
    }

    ui.separator();
    if ui.button("Clear joint selection").clicked() {
        state.document.joint_selection = None;
    }
}