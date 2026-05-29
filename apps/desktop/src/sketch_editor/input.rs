use bevy::prelude::*;
use crate::camera::OrbitCamera;
use kpe_geometry::sketch::entities::{EntityId, closest_point_on_line};
use kpe_geometry::sketch::constraints::Constraint;
use kpe_geometry::sketch::spatial::{SpatialIndex, SpatialEntity};
use super::state::{SketchEditorState, SketchTool};
use super::math::{to_2d, sketch_plane_normal};
use super::solver;

fn ray_plane_intersection(ray_origin: Vec3, ray_dir: Vec3, plane: &kpe_schema::geometry::SketchPlane) -> Option<Vec3> {
    let normal = sketch_plane_normal(plane).as_vec3();
    let denom = ray_dir.dot(normal);
    if denom.abs() < 1e-6 { return None; }
    let t = -ray_origin.dot(normal) / denom;
    if t < 0.0 { return None; }
    Some(ray_origin + ray_dir * t)
}

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
    let (x, y) = to_2d(hit, &editor.plane);
    let (x, y) = if editor.grid_snap {
        let s = editor.snap_size;
        ((x / s).round() * s, (y / s).round() * s)
    } else {
        (x, y)
    };

    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if ctrl && keys.just_pressed(KeyCode::KeyZ) {
        if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
            editor.redo();
        } else {
            editor.undo();
        }
        return;
    }
    if ctrl && keys.just_pressed(KeyCode::KeyY) {
        editor.redo();
        return;
    }
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
        return;
    }

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
                    for c in inferred {
                        editor.document.add_constraint(c);
                    }
                    solver::solve_sync(&mut editor);
                    editor.line_start = None;
                    editor.selected_entity = Some(lid);
                } else {
                    editor.line_start = Some((x, y));
                }
            }
            SketchTool::Select => {
                let idx = SpatialIndex::build(
                    &editor.document.points,
                    &editor.document.lines,
                    &editor.document.arcs,
                    &editor.document.circles,
                    0.5,
                );

                // Point pick (threshold 0.2)
                if let Some((pid, _)) = idx.nearest_point(x, y, 0.2, &editor.document.points) {
                    if ctrl {
                        if editor.selected_entities.contains(&pid) {
                            editor.selected_entities.retain(|&id| id != pid);
                        } else {
                            editor.selected_entities.push(pid);
                        }
                        return;
                    }
                    let pos = editor.document.points.iter().find(|p| p.id == pid).map(|p| (p.x, p.y)).unwrap_or((x, y));
                    editor.drag_from = Some(pid);
                    editor.drag_offset = glam::DVec2::new(x, y) - glam::DVec2::new(pos.0, pos.1);
                    return;
                }

                // Line / Circle / Arc pick (threshold 0.15)
                let candidates = idx.query(x, y, 0.15);
                let pa = glam::DVec2::new(x, y);
                let mut best = None;
                let mut best_d = 0.15f64;
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
                                    let d_to_perim = (pa.distance(glam::DVec2::new(center.x, center.y)) - c.radius).abs();
                                    if d_to_perim < best_d { best_d = d_to_perim; best = Some(cid); }
                                }
                            }
                        }
                        SpatialEntity::Arc(aid) => {
                            if let Some(a) = editor.document.arcs.iter().find(|a| a.id == aid) {
                                if let Some(center) = editor.document.points.iter().find(|p| p.id == a.center) {
                                    let d_to_perim = (pa.distance(glam::DVec2::new(center.x, center.y)) - a.radius).abs();
                                    if d_to_perim < best_d { best_d = d_to_perim; best = Some(aid); }
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
                        } else {
                            editor.selected_entities.push(id);
                        }
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
                    let r = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
                    if r > 0.01 {
                        editor.save_snapshot();
                        let id = editor.document.add_circle(cx, cy, r);
                        solver::solve_sync(&mut editor);
                        editor.selected_entity = Some(id);
                    }
                    editor.circle_center = None;
                } else {
                    editor.circle_center = Some((x, y));
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
                } else {
                    editor.arc_center = Some((x, y));
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
