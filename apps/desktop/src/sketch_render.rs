use bevy::prelude::*;
use crate::app::AppState;
use crate::sketch_editor::SketchEditorState;
use crate::sketch_editor::{to_3d, sketch_plane_normal, circle_basis};
use kpe_schema::geometry::*;
use kpe_geometry::sketch::tessellate_sketch;
use kpe_geometry::sketch::constraints::Constraint;
use kpe_geometry::sketch::document::SketchDocument;
use kpe_geometry::sketch::entities::{EntityId, Point};

pub fn render_sketch_wireframes(
    mut gizmos: Gizmos,
    state: Res<AppState>,
    editor: Res<SketchEditorState>,
) {
    if editor.active { return; }
    draw_sketch_node_wireframes(&state.document.recipe.scene, &mut gizmos);
}

fn draw_sketch_node_wireframes(node: &GeometryNode, gizmos: &mut Gizmos) {
    if let GeometryNodeType::Sketch(sketch_def) = &node.node_type {
        let contours = tessellate_sketch(sketch_def);
        let plane = &sketch_def.plane;
        let color = Color::srgb(0.2, 0.8, 0.2);

        for contour in &contours {
            for i in 0..contour.len() {
                let j = (i + 1) % contour.len();
                let a = match plane {
                    SketchPlane::XY => Vec3::new(contour[i].x as f32, contour[i].y as f32, 0.0),
                    SketchPlane::XZ => Vec3::new(contour[i].x as f32, 0.0, contour[i].y as f32),
                    SketchPlane::YZ => Vec3::new(0.0, contour[i].x as f32, contour[i].y as f32),
                };
                let b = match plane {
                    SketchPlane::XY => Vec3::new(contour[j].x as f32, contour[j].y as f32, 0.0),
                    SketchPlane::XZ => Vec3::new(contour[j].x as f32, 0.0, contour[j].y as f32),
                    SketchPlane::YZ => Vec3::new(0.0, contour[j].x as f32, contour[j].y as f32),
                };
                gizmos.line(a, b, color);
            }
        }
    }
    for child in &node.children {
        draw_sketch_node_wireframes(child, gizmos);
    }
}

pub fn render_sketch(
    mut gizmos: Gizmos,
    editor: Res<SketchEditorState>,
) {
    if !editor.active { return; }

    let plane = &editor.plane;
    let grid_size = editor.grid_size as f32;
    let range = 5.0;
    let steps = (range / grid_size) as i32;
    let mut i = -steps;
    while i <= steps {
        let v = i as f32 * grid_size;
        let (a, b) = match plane {
            SketchPlane::XY => (Vec3::new(v, -range, 0.0), Vec3::new(v, range, 0.0)),
            SketchPlane::XZ => (Vec3::new(v, 0.0, -range), Vec3::new(v, 0.0, range)),
            SketchPlane::YZ => (Vec3::new(0.0, v, -range), Vec3::new(0.0, v, range)),
        };
        let (c, d) = match plane {
            SketchPlane::XY => (Vec3::new(-range, v, 0.0), Vec3::new(range, v, 0.0)),
            SketchPlane::XZ => (Vec3::new(-range, 0.0, v), Vec3::new(range, 0.0, v)),
            SketchPlane::YZ => (Vec3::new(0.0, -range, v), Vec3::new(0.0, range, v)),
        };
        let color = if i == 0 { Color::srgb(0.5, 0.5, 0.5) } else { Color::srgb(0.25, 0.25, 0.25) };
        gizmos.line(a, b, color);
        gizmos.line(c, d, color);
        i += 1;
    }

    let normal = sketch_plane_normal(plane);
    let (right, forward) = circle_basis(normal);

    for p in &editor.document.points {
        let pos = to_3d(p.x, p.y, plane);
        let color = entity_dof_color(
            &[p.id],
            &editor.document.points,
            &editor.dof_status,
            editor.drag_from == Some(p.id),
            Color::srgb(1.0, 1.0, 0.0),
            Color::srgb(0.2, 0.8, 1.0),
        );
        gizmos.sphere(pos, 0.05, color);
    }

    for l in &editor.document.lines {
        if let (Some(s), Some(e)) = (
            editor.document.points.iter().find(|p| p.id == l.start),
            editor.document.points.iter().find(|p| p.id == l.end),
        ) {
            let a = to_3d(s.x, s.y, plane);
            let b = to_3d(e.x, e.y, plane);
            let color = entity_dof_color(
                &[s.id, e.id],
                &editor.document.points,
                &editor.dof_status,
                editor.selected_entities.contains(&l.id) || Some(l.id) == editor.selected_entity,
                Color::srgb(1.0, 0.8, 0.0),
                Color::srgb(0.0, 0.8, 1.0),
            );
            gizmos.line(a, b, color);
        }
    }

    for c in &editor.document.circles {
        if let Some(center) = editor.document.points.iter().find(|p| p.id == c.center) {
            let pos = to_3d(center.x, center.y, plane);
            let color = entity_dof_color(
                &[center.id],
                &editor.document.points,
                &editor.dof_status,
                editor.selected_entities.contains(&c.id) || Some(c.id) == editor.selected_entity,
                Color::srgb(1.0, 0.8, 0.0),
                Color::srgb(0.0, 0.8, 1.0),
            );
            draw_circle_lines(&mut gizmos, pos, right, forward, c.radius as f32, color);
        }
    }

    for a in &editor.document.arcs {
        if let Some(center) = editor.document.points.iter().find(|p| p.id == a.center) {
            let pos = to_3d(center.x, center.y, plane);
            let color = entity_dof_color(
                &[center.id],
                &editor.document.points,
                &editor.dof_status,
                editor.selected_entities.contains(&a.id) || Some(a.id) == editor.selected_entity,
                Color::srgb(1.0, 0.8, 0.0),
                Color::srgb(0.2, 1.0, 0.2),
            );
            let segs = 16;
            let radius = a.radius as f32;
            for i in 0..segs {
                let t0 = i as f32 / segs as f32;
                let t1 = (i + 1) as f32 / segs as f32;
                let a0 = t0 * a.sweep_angle as f32;
                let a1 = t1 * a.sweep_angle as f32;
                let p0 = pos + right * a0.cos() * radius + forward * a0.sin() * radius;
                let p1 = pos + right * a1.cos() * radius + forward * a1.sin() * radius;
                gizmos.line(p0, p1, color);
            }
        }
    }

    if editor.show_constraints {
        for c in &editor.document.constraints {
            let pos = constraint_pos(c, &editor.document, plane);
            if let Some(p) = pos {
                gizmos.sphere(p, 0.04, Color::srgb(1.0, 0.5, 0.0));
            }
        }
    }

    if let Some((sx, sy)) = editor.line_start {
        let a = to_3d(sx, sy, plane);
        gizmos.line(a, a + Vec3::new(0.0, 0.2, 0.0), Color::srgb(0.0, 1.0, 0.0));
        gizmos.sphere(a, 0.1, Color::srgb(0.0, 1.0, 0.0));
    }

    if let Some((cx, cy)) = editor.circle_center {
        let pos = to_3d(cx, cy, plane);
        gizmos.sphere(pos, 0.08, Color::srgb(1.0, 0.5, 0.0));
        gizmos.line(pos - Vec3::X * 0.15, pos + Vec3::X * 0.15, Color::srgb(1.0, 0.5, 0.0));
        gizmos.line(pos - Vec3::Y * 0.15, pos + Vec3::Y * 0.15, Color::srgb(1.0, 0.5, 0.0));
    }

    if let Some((cx, cy)) = editor.arc_center {
        let pos = to_3d(cx, cy, plane);
        gizmos.sphere(pos, 0.08, Color::srgb(1.0, 0.3, 0.3));
        gizmos.line(pos - Vec3::X * 0.15, pos + Vec3::X * 0.15, Color::srgb(1.0, 0.3, 0.3));
        gizmos.line(pos - Vec3::Y * 0.15, pos + Vec3::Y * 0.15, Color::srgb(1.0, 0.3, 0.3));
    }

    if let Some((ax, ay)) = editor.measure_click_a {
        let pa = to_3d(ax, ay, plane);
        gizmos.sphere(pa, 0.08, Color::srgb(0.0, 1.0, 0.0));
        if let Some((bx, by)) = editor.measure_click_b {
            let pb = to_3d(bx, by, plane);
            gizmos.line(pa, pb, Color::srgb(0.0, 1.0, 0.0));
            let mid = (pa + pb) * 0.5;
            gizmos.sphere(mid, 0.06, Color::srgb(0.0, 1.0, 0.0));
        }
    }
}

fn draw_circle_lines(gizmos: &mut Gizmos, center: Vec3, right: Vec3, forward: Vec3, radius: f32, color: Color) {
    let segs = 32;
    for i in 0..segs {
        let a0 = (i as f32 / segs as f32) * std::f32::consts::TAU;
        let a1 = ((i + 1) as f32 / segs as f32) * std::f32::consts::TAU;
        let p0 = center + right * a0.cos() * radius + forward * a0.sin() * radius;
        let p1 = center + right * a1.cos() * radius + forward * a1.sin() * radius;
        gizmos.line(p0, p1, color);
    }
}

/// Check whether all points referenced by an entity are fully constrained.
/// Returns `None` when DoF data is unavailable (e.g. empty / stale).
fn entity_dof_color(
    point_ids: &[EntityId],
    points: &[Point],
    dof_status: &[(bool, bool)],
    selected: bool,
    selected_color: Color,
    default_color: Color,
) -> Color {
    if selected {
        return selected_color;
    }
    if dof_status.len() != points.len() {
        return default_color;
    }
    let all_fully = point_ids.iter().all(|id| {
        points
            .iter()
            .position(|p| p.id == *id)
            .and_then(|i| dof_status.get(i))
            .map_or(false, |&(x, y)| x && y)
    });
    if all_fully {
        Color::BLACK
    } else {
        default_color
    }
}

fn constraint_pos(c: &Constraint, doc: &SketchDocument, plane: &SketchPlane) -> Option<Vec3> {
    match *c {
        Constraint::Horizontal { line } | Constraint::Vertical { line } => {
            let l = doc.lines.iter().find(|l| l.id == line)?;
            let s = doc.points.iter().find(|p| p.id == l.start)?;
            let e = doc.points.iter().find(|p| p.id == l.end)?;
            Some(to_3d((s.x + e.x) * 0.5, (s.y + e.y) * 0.5, plane) + Vec3::Y * 0.35)
        }
        Constraint::Distance { point_a, point_b, .. } => {
            let pa = doc.points.iter().find(|p| p.id == point_a)?;
            let pb = doc.points.iter().find(|p| p.id == point_b)?;
            Some(to_3d((pa.x + pb.x) * 0.5, (pa.y + pb.y) * 0.5, plane) + Vec3::Y * 0.35)
        }
        Constraint::EqualLength { line_a, .. } => {
            let l = doc.lines.iter().find(|l| l.id == line_a)?;
            let s = doc.points.iter().find(|p| p.id == l.start)?;
            let e = doc.points.iter().find(|p| p.id == l.end)?;
            Some(to_3d((s.x + e.x) * 0.5, (s.y + e.y) * 0.5, plane) + Vec3::Y * 0.35)
        }
        Constraint::Parallel { line_a, .. } | Constraint::Perpendicular { line_a, .. } | Constraint::Collinear { line_a, .. } => {
            let l = doc.lines.iter().find(|l| l.id == line_a)?;
            let s = doc.points.iter().find(|p| p.id == l.start)?;
            let e = doc.points.iter().find(|p| p.id == l.end)?;
            Some(to_3d((s.x + e.x) * 0.5, (s.y + e.y) * 0.5, plane) + Vec3::Y * 0.35)
        }
        Constraint::Coincident { point_a, .. } => {
            let p = doc.points.iter().find(|p| p.id == point_a)?;
            Some(to_3d(p.x, p.y, plane) + Vec3::new(0.25, 0.25, 0.0))
        }
        Constraint::Fix { point, .. } => {
            let p = doc.points.iter().find(|p| p.id == point)?;
            Some(to_3d(p.x, p.y, plane) - Vec3::Y * 0.35)
        }
        _ => None,
    }
}

pub fn constraint_marker_pos(c: &Constraint, doc: &SketchDocument) -> (f64, f64) {
    match *c {
        Constraint::Distance { point_a, point_b, .. } => {
            let pa = doc.points.iter().find(|p| p.id == point_a);
            let pb = doc.points.iter().find(|p| p.id == point_b);
            if let (Some(pa), Some(pb)) = (pa, pb) {
                ((pa.x + pb.x) * 0.5, (pa.y + pb.y) * 0.5)
            } else {
                (0.0, 0.0)
            }
        }
        Constraint::Angle { line_a, line_b, .. } => {
            let la = doc.lines.iter().find(|l| l.id == line_a);
            let lb = doc.lines.iter().find(|l| l.id == line_b);
            if let (Some(la), Some(lb)) = (la, lb) {
                let sa = doc.points.iter().find(|p| p.id == la.start);
                let ea = doc.points.iter().find(|p| p.id == la.end);
                let sb = doc.points.iter().find(|p| p.id == lb.start);
                let eb = doc.points.iter().find(|p| p.id == lb.end);
                let sx = sa.map_or(0.0, |p| p.x) + sb.map_or(0.0, |p| p.x)
                    + ea.map_or(0.0, |p| p.x) + eb.map_or(0.0, |p| p.x);
                let sy = sa.map_or(0.0, |p| p.y) + sb.map_or(0.0, |p| p.y)
                    + ea.map_or(0.0, |p| p.y) + eb.map_or(0.0, |p| p.y);
                (sx * 0.25, sy * 0.25)
            } else {
                (0.0, 0.0)
            }
        }
        Constraint::Radius { .. } => {
            (0.0, 0.0)
        }
        ref c @ (Constraint::Horizontal { .. } | Constraint::Vertical { .. }) => {
            let line = match *c {
                Constraint::Horizontal { line } | Constraint::Vertical { line } => line,
                _ => unreachable!(),
            };
            doc.lines.iter().find(|l| l.id == line).and_then(|l| {
                let s = doc.points.iter().find(|p| p.id == l.start)?;
                let e = doc.points.iter().find(|p| p.id == l.end)?;
                Some(((s.x + e.x) * 0.5, (s.y + e.y) * 0.5))
            }).unwrap_or((0.0, 0.0))
        }
        Constraint::EqualLength { line_a, .. } | Constraint::Parallel { line_a, .. }
            | Constraint::Perpendicular { line_a, .. } | Constraint::Collinear { line_a, .. } => {
            doc.lines.iter().find(|l| l.id == line_a).and_then(|l| {
                let s = doc.points.iter().find(|p| p.id == l.start)?;
                let e = doc.points.iter().find(|p| p.id == l.end)?;
                Some(((s.x + e.x) * 0.5, (s.y + e.y) * 0.5))
            }).unwrap_or((0.0, 0.0))
        }
        Constraint::Coincident { point_a, .. } => {
            if let Some(p) = doc.points.iter().find(|pt| pt.id == point_a) {
                (p.x + 0.25, p.y + 0.25)
            } else {
                (0.0, 0.0)
            }
        }
        Constraint::Fix { point, .. } => {
            if let Some(p) = doc.points.iter().find(|pt| pt.id == point) {
                (p.x, p.y - 0.35)
            } else {
                (0.0, 0.0)
            }
        }
        _ => (0.0, 0.0),
    }
}


