use bevy::prelude::*;
use crate::camera::OrbitCamera;
use kpe_geometry::sketch::entities::{EntityId, closest_point_on_line};
use kpe_geometry::sketch::constraints::Constraint;
use kpe_geometry::sketch::spatial::{SpatialIndex, SpatialEntity};
use kpe_geometry::sketch::osnap::{OsnapEngine, SnapFilter};
use super::state::{SketchEditorState, SketchTool, ToolPhase};
use super::math::{to_2d, sketch_plane_normal};
use super::solver;

// ── Helpers ──────────────────────────────────────────────────────────

fn ray_plane_intersection(
    ray_origin: Vec3, ray_dir: Vec3, plane: &kpe_schema::geometry::SketchPlane,
) -> Option<Vec3> {
    let normal = sketch_plane_normal(plane).as_vec3();
    let denom = ray_dir.dot(normal);
    if denom.abs() < 1e-6 { return None; }
    let t = -ray_origin.dot(normal) / denom;
    if t < 0.0 { return None; }
    Some(ray_origin + ray_dir * t)
}

fn osnap_filter(editor: &SketchEditorState) -> SnapFilter {
    SnapFilter {
        endpoint:     editor.osnap_settings.endpoint,
        midpoint:     editor.osnap_settings.midpoint,
        center:       editor.osnap_settings.center,
        quadrant:     editor.osnap_settings.quadrant,
        intersection: editor.osnap_settings.intersection,
        on_entity:    editor.osnap_settings.on_entity,
    }
}

fn apply_ortho(ox: f64, oy: f64, x: &mut f64, y: &mut f64) {
    let abs_dx = (*x - ox).abs();
    let abs_dy = (*y - oy).abs();
    if abs_dx >= abs_dy { *y = oy } else { *x = ox }
}

/// Determine which origin to use for angle-based computations.
fn get_origin(editor: &SketchEditorState) -> Option<(f64, f64)> {
    editor.line_start
        .or(editor.circle_center)
        .or(editor.arc_center)
}

/// Find the nearest entity under the cursor for hover highlighting.
fn find_hover(
    x: f64, y: f64, editor: &SketchEditorState,
) -> Option<EntityId> {
    let idx = SpatialIndex::build(
        &editor.document.points,
        &editor.document.lines,
        &editor.document.arcs,
        &editor.document.circles,
        0.5,
    );
    // Check points first (threshold 0.2)
    if let Some((pid, _)) = idx.nearest_point(x, y, 0.2, &editor.document.points) {
        return Some(pid);
    }
    // Check lines/circles/arcs (threshold 0.15)
    let pa = glam::DVec2::new(x, y);
    let mut best = None;
    let mut best_d = 0.15;
    for e in idx.query(x, y, 0.15) {
        match e {
            SpatialEntity::Line(lid) => {
                if let Some(l) = editor.document.lines.iter().find(|l| l.id == lid) {
                    if let (Some(s), Some(e)) = (
                        editor.document.points.iter().find(|p| p.id == l.start),
                        editor.document.points.iter().find(|p| p.id == l.end),
                    ) {
                        let closest = closest_point_on_line(pa, glam::DVec2::new(s.x, s.y), glam::DVec2::new(e.x, e.y));
                        let d = pa.distance(closest);
                        if d < best_d { best_d = d; best = Some(lid); }
                    }
                }
            }
            SpatialEntity::Circle(cid) => {
                if let Some(c) = editor.document.circles.iter().find(|c| c.id == cid) {
                    if let Some(center) = editor.document.points.iter().find(|p| p.id == c.center) {
                        let d = (pa.distance(glam::DVec2::new(center.x, center.y)) - c.radius).abs();
                        if d < best_d { best_d = d; best = Some(cid); }
                    }
                }
            }
            SpatialEntity::Arc(aid) => {
                if let Some(a) = editor.document.arcs.iter().find(|a| a.id == aid) {
                    if let Some(center) = editor.document.points.iter().find(|p| p.id == a.center) {
                        let d = (pa.distance(glam::DVec2::new(center.x, center.y)) - a.radius).abs();
                        if d < best_d { best_d = d; best = Some(aid); }
                    }
                }
            }
            _ => {}
        }
    }
    best
}

// ── Main system ──────────────────────────────────────────────────────

pub fn sketch_input(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<OrbitCamera>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut editor: ResMut<SketchEditorState>,
) {
    if !editor.active { return; }

    let window = windows.single();
    let cursor = match window.cursor_position() {
        Some(c) => c,
        None => return,
    };
    let (cam, cam_transform) = match cameras.get_single() {
        Ok(c) => c,
        Err(_) => return,
    };
    let Ok(ray) = cam.viewport_to_world(cam_transform, cursor) else { return };
    let Some(hit) = ray_plane_intersection(ray.origin, ray.direction.as_vec3(), &editor.plane) else { return };
    let (mut x, mut y) = to_2d(hit, &editor.plane);

    // ── Grid snap ────────────────────────────────────────────────────
    if editor.grid_snap {
        let s = editor.snap_size;
        x = (x / s).round() * s;
        y = (y / s).round() * s;
    }

    // ── Ortho constraint (F8) ───────────────────────────────────────
    if editor.ortho_enabled {
        if let Some((ox, oy)) = get_origin(&editor) {
            apply_ortho(ox, oy, &mut x, &mut y);
        }
    }

    // ── DynamicInput resolve ─────────────────────────────────────────
    if editor.dynamic_input.is_active() {
        let origin = get_origin(&editor).unwrap_or((x, y));
        let result = if editor.polar_enabled {
            editor.dynamic_input.resolve(
                glam::DVec2::new(origin.0, origin.1),
                glam::DVec2::new(x, y),
                Some(&editor.polar_snap),
            )
        } else {
            editor.dynamic_input.resolve(
                glam::DVec2::new(origin.0, origin.1),
                glam::DVec2::new(x, y),
                None,
            )
        };
        x = result.point.x;
        y = result.point.y;
    }

    // ── OSnap ────────────────────────────────────────────────────────
    let filter = osnap_filter(&editor);
    let snap_point = if filter.any_enabled() {
        OsnapEngine::snap_filtered(
            glam::DVec2::new(x, y),
            &editor.document.points,
            &editor.document.lines,
            &editor.document.arcs,
            &editor.document.circles,
            &filter,
        )
    } else {
        None
    };
    if let Some(ref s) = snap_point {
        x = s.x;
        y = s.y;
    }
    editor.snap_feedback = snap_point;

    // ── Hover detection (every frame) ───────────────────────────────
    if editor.tool == SketchTool::Select {
        let hovered = find_hover(x, y, &editor);
        editor.hovered_entity = hovered;
    } else {
        editor.hovered_entity = None;
    }

    // ── Update cursor tooltip ───────────────────────────────────────
    let tooltip = match editor.tool {
        SketchTool::Select => {
            if editor.marquee_start.is_some() {
                "Drag to select entities".into()
            } else if editor.drag_from.is_some() {
                "Drag to move point".into()
            } else { "Click to select. Drag for marquee. Ctrl+click to toggle.".into() }
        }
        SketchTool::Line => match editor.tool_phase {
            ToolPhase::Idle => "Specify first point (Esc to cancel)".into(),
            ToolPhase::Place => "Specify next point (Esc to finish chain)".into(),
        },
        SketchTool::Circle => {
            if editor.circle_center.is_some() { "Specify radius (or click to set)".into() }
            else { "Specify circle center".into() }
        }
        SketchTool::Arc => {
            if editor.arc_center.is_some() { "Specify arc angle (click to set)".into() }
            else { "Specify arc center".into() }
        }
        SketchTool::Measure => {
            if editor.measure_click_a.is_some() { "Specify second point".into() }
            else { "Specify first point".into() }
        }
    };
    editor.cursor_tooltip = tooltip;

    // ── Keyboard shortcuts ───────────────────────────────────────────
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    if keys.just_pressed(KeyCode::Escape) {
        editor.line_start = None;
        editor.circle_center = None;
        editor.arc_center = None;
        editor.drag_from = None;
        editor.drag_start_saved = false;
        editor.measure_click_a = None;
        editor.measure_click_b = None;
        editor.measure_result = None;
        editor.tool_phase = ToolPhase::Idle;
        editor.snap_feedback = None;
        editor.hovered_entity = None;
        editor.cursor_tooltip.clear();
        editor.marquee_start = None;
        editor.marquee_end = None;
        editor.dynamic_input.clear();
        return;
    }

    if keys.just_pressed(KeyCode::F8) {
        editor.ortho_enabled = !editor.ortho_enabled;
    }

    if ctrl && keys.just_pressed(KeyCode::KeyZ) {
        if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
            editor.redo();
        } else { editor.undo(); }
        return;
    }
    if ctrl && keys.just_pressed(KeyCode::KeyY) { editor.redo(); return; }
    if ctrl && keys.just_pressed(KeyCode::KeyA) {
        let mut all_ids: Vec<EntityId> = editor.document.lines.iter().map(|l| l.id).collect();
        all_ids.extend(editor.document.circles.iter().map(|c| c.id));
        all_ids.extend(editor.document.arcs.iter().map(|a| a.id));
        editor.selected_entities = all_ids;
        return;
    }

    if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
        editor.delete_selected();
        return;
    }

    if mouse.just_pressed(MouseButton::Right) {
        editor.line_start = None;
        editor.drag_from = None;
        editor.drag_start_saved = false;
        editor.circle_center = None;
        editor.arc_center = None;
        editor.measure_click_a = None;
        editor.measure_click_b = None;
        editor.measure_result = None;
        editor.tool_phase = ToolPhase::Idle;
        editor.snap_feedback = None;
        editor.hovered_entity = None;
        editor.cursor_tooltip.clear();
        editor.marquee_start = None;
        editor.marquee_end = None;
        editor.dynamic_input.clear();
        return;
    }

    // ── Marquee update (every frame while dragging) ─────────────────
    if editor.marquee_start.is_some() && mouse.pressed(MouseButton::Left) {
        editor.marquee_end = Some((x, y));
    }

    // ── Left-click actions ───────────────────────────────────────────
    if mouse.just_pressed(MouseButton::Left) {
        match editor.tool {
            SketchTool::Line => {
                if let Some((sx, sy)) = editor.line_start {
                    editor.save_snapshot();
                    let p1 = editor.document.add_point(sx, sy);
                    let p2 = editor.document.add_point(x, y);
                    let lid = editor.document.add_line(p1, p2);
                    let inferred = editor.document.infer_constraints(
                        lid, glam::DVec2::new(sx, sy), glam::DVec2::new(x, y));
                    for c in inferred { editor.document.add_constraint(c); }
                    solver::solve_sync(&mut editor);
                    editor.selected_entity = Some(lid);
                    editor.line_start = Some((x, y));
                    editor.tool_phase = ToolPhase::Place;
                } else {
                    editor.line_start = Some((x, y));
                    editor.tool_phase = ToolPhase::Idle;
                }
            }
            SketchTool::Select => {
                // If marquee is active (dragging), this is the release → compute selection
                if let Some((sx, sy)) = editor.marquee_start {
                    let ex = x;
                    let ey = y;
                    let is_crossing = sx > ex; // R→L drag = crossing
                    let min_x = sx.min(ex);
                    let max_x = sx.max(ex);
                    let min_y = sy.min(ey);
                    let max_y = sy.max(ey);

                    let mut selected = Vec::new();
                    // Test points
                    for p in &editor.document.points {
                        let inside = if is_crossing {
                            // Crossing: any entity that touches the rect
                            p.x >= min_x - 0.1 && p.x <= max_x + 0.1
                                && p.y >= min_y - 0.1 && p.y <= max_y + 0.1
                        } else {
                            // Window: fully inside
                            p.x >= min_x && p.x <= max_x && p.y >= min_y && p.y <= max_y
                        };
                        if inside { selected.push(p.id); }
                    }
                    // Test lines (check if both endpoints inside for window, or either for crossing)
                    for l in &editor.document.lines {
                        if let (Some(s), Some(e)) = (
                            editor.document.points.iter().find(|p| p.id == l.start),
                            editor.document.points.iter().find(|p| p.id == l.end),
                        ) {
                            let both = s.x >= min_x && s.x <= max_x && s.y >= min_y && s.y <= max_y
                                && e.x >= min_x && e.x <= max_x && e.y >= min_y && e.y <= max_y;
                            let either = (s.x >= min_x && s.x <= max_x && s.y >= min_y && s.y <= max_y)
                                || (e.x >= min_x && e.x <= max_x && e.y >= min_y && e.y <= max_y);
                            if (is_crossing && either) || (!is_crossing && both) {
                                selected.push(l.id);
                            }
                        }
                    }
                    editor.selected_entities = selected;
                    editor.marquee_start = None;
                    editor.marquee_end = None;
                    return;
                }

                // Normal select (no marquee active)
                let idx = SpatialIndex::build(
                    &editor.document.points,
                    &editor.document.lines,
                    &editor.document.arcs,
                    &editor.document.circles,
                    0.5,
                );

                if let Some((pid, _)) = idx.nearest_point(x, y, 0.2, &editor.document.points) {
                    if ctrl {
                        if editor.selected_entities.contains(&pid) {
                            editor.selected_entities.retain(|&id| id != pid);
                        } else { editor.selected_entities.push(pid); }
                        return;
                    }
                    let pos = editor.document.points.iter()
                        .find(|p| p.id == pid).map(|p| (p.x, p.y)).unwrap_or((x, y));
                    editor.drag_from = Some(pid);
                    editor.drag_offset = glam::DVec2::new(x, y) - glam::DVec2::new(pos.0, pos.1);
                    return;
                }

                let candidates = idx.query(x, y, 0.15);
                let pa = glam::DVec2::new(x, y);
                let mut best = None;
                let mut best_d = 0.15;
                for e in &candidates {
                    match *e {
                        SpatialEntity::Line(lid) => {
                            if let Some(l) = editor.document.lines.iter().find(|l| l.id == lid) {
                                if let (Some(s), Some(e)) = (
                                    editor.document.points.iter().find(|p| p.id == l.start),
                                    editor.document.points.iter().find(|p| p.id == l.end),
                                ) {
                                    let closest = closest_point_on_line(pa, glam::DVec2::new(s.x, s.y), glam::DVec2::new(e.x, e.y));
                                    let d = pa.distance(closest);
                                    if d < best_d { best_d = d; best = Some(lid); }
                                }
                            }
                        }
                        SpatialEntity::Circle(cid) => {
                            if let Some(c) = editor.document.circles.iter().find(|c| c.id == cid) {
                                if let Some(center) = editor.document.points.iter().find(|p| p.id == c.center) {
                                    let d = (pa.distance(glam::DVec2::new(center.x, center.y)) - c.radius).abs();
                                    if d < best_d { best_d = d; best = Some(cid); }
                                }
                            }
                        }
                        SpatialEntity::Arc(aid) => {
                            if let Some(a) = editor.document.arcs.iter().find(|a| a.id == aid) {
                                if let Some(center) = editor.document.points.iter().find(|p| p.id == a.center) {
                                    let d = (pa.distance(glam::DVec2::new(center.x, center.y)) - a.radius).abs();
                                    if d < best_d { best_d = d; best = Some(aid); }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                if ctrl {
                    if let Some(id) = best {
                        if editor.selected_entities.contains(&id) {
                            editor.selected_entities.retain(|&x| x != id);
                        } else { editor.selected_entities.push(id); }
                    }
                } else {
                    if best.is_none() {
                        let pick: Vec<(usize, (f64, f64))> = editor.document.constraints.iter().enumerate()
                            .map(|(ci, c)| (ci, crate::sketch_render::constraint_marker_pos(c, &editor.document))).collect();
                        for (ci, pos) in &pick {
                            let d = glam::DVec2::new(x - pos.0, y - pos.1).length();
                            if d < 0.15 {
                                let val = editor.document.constraints.get(*ci).map(|c| match *c {
                                    Constraint::Distance { distance, .. } => distance,
                                    Constraint::Angle { angle, .. } => angle,
                                    Constraint::Radius { radius, .. } => radius,
                                    _ => 0.0,
                                }).unwrap_or(0.0);
                                editor.editing_constraint_idx = Some(*ci);
                                editor.editing_new_value = val;
                                break;
                            }
                        }
                    }
                    editor.selected_entity = best;
                }
            }
            SketchTool::Circle => {
                if let Some((cx, cy)) = editor.circle_center {
                    let r = if editor.circle_diameter_mode {
                        ((x - cx).powi(2) + (y - cy).powi(2)).sqrt() * 0.5
                    } else {
                        ((x - cx).powi(2) + (y - cy).powi(2)).sqrt()
                    };
                    if r > 0.01 {
                        editor.save_snapshot();
                        let id = editor.document.add_circle(cx, cy, r);
                        solver::solve_sync(&mut editor);
                        editor.selected_entity = Some(id);
                    }
                    editor.circle_center = None;
                    editor.tool_phase = ToolPhase::Idle;
                } else {
                    editor.circle_center = Some((x, y));
                    editor.tool_phase = ToolPhase::Place;
                }
            }
            SketchTool::Arc => {
                if let Some((cx, cy)) = editor.arc_center {
                    let r = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
                    if r > 0.01 {
                        editor.save_snapshot();
                        let angle = (y - cy).atan2(x - cx);
                        let id = editor.document.add_arc(cx, cy, r, 0.0, angle);
                        solver::solve_sync(&mut editor);
                        editor.selected_entity = Some(id);
                    }
                    editor.arc_center = None;
                    editor.tool_phase = ToolPhase::Idle;
                } else {
                    editor.arc_center = Some((x, y));
                    editor.tool_phase = ToolPhase::Place;
                }
            }
            SketchTool::Measure => {
                if let Some((ax, ay)) = editor.measure_click_a {
                    let dx = x - ax;
                    let dy = y - ay;
                    let dist = (dx * dx + dy * dy).sqrt();
                    let angle = dy.atan2(dx).to_degrees();
                    editor.measure_click_b = Some((x, y));
                    editor.measure_result = Some(format!("d: {:.3}  \u{2220}: {:.1}\u{b0}", dist, angle));
                } else {
                    editor.measure_click_a = Some((x, y));
                    editor.measure_click_b = None;
                    editor.measure_result = None;
                }
            }
        }
    }

    // ── Start marquee on plain left-click-drag in Select tool ────────
    if editor.tool == SketchTool::Select
        && editor.marquee_start.is_none()
        && mouse.just_pressed(MouseButton::Left)
        && editor.drag_from.is_none()
    {
        // Only start marquee if we didn't click on an entity
        let idx = SpatialIndex::build(
            &editor.document.points,
            &editor.document.lines,
            &editor.document.arcs,
            &editor.document.circles,
            0.5,
        );
        let has_pick = idx.nearest_point(x, y, 0.2, &editor.document.points).is_some()
            || !idx.query(x, y, 0.15).is_empty();
        if !has_pick {
            editor.marquee_start = Some((x, y));
            editor.marquee_end = Some((x, y));
        }
    }

    // ── Drag selected point ──────────────────────────────────────────
    if let Some(pid) = editor.drag_from {
        if mouse.just_released(MouseButton::Left) {
            editor.drag_from = None;
        } else if mouse.pressed(MouseButton::Left) && !editor.drag_start_saved {
            editor.save_snapshot();
            editor.drag_start_saved = true;
            let offset = editor.drag_offset;
            if let Some(p) = editor.document.points.iter_mut().find(|p| p.id == pid) {
                p.x = x - offset.x;
                p.y = y - offset.y;
                solver::solve_sync(&mut editor);
            }
        } else if mouse.pressed(MouseButton::Left) {
            let offset = editor.drag_offset;
            if let Some(p) = editor.document.points.iter_mut().find(|p| p.id == pid) {
                p.x = x - offset.x;
                p.y = y - offset.y;
                solver::solve_sync(&mut editor);
            }
        }
    }
}
