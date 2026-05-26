use glam::{DVec2, DVec3, DQuat, DMat3};
use kpe_schema::geometry::{
    ExtrudeDef, RevolveDef, RevolveAxis, SweepDef, SweepPath, SketchDef, SketchPlane, TriangleMesh,
};
use crate::sketch::tessellate_sketch;

fn project_to_3d(p: DVec2, plane: &SketchPlane) -> DVec3 {
    match plane {
        SketchPlane::XY => DVec3::new(p.x, p.y, 0.0),
        SketchPlane::XZ => DVec3::new(p.x, 0.0, p.y),
        SketchPlane::YZ => DVec3::new(0.0, p.y, p.x),
    }
}

fn extrude_direction(plane: &SketchPlane) -> DVec3 {
    match plane {
        SketchPlane::XY => DVec3::Z,
        SketchPlane::XZ => DVec3::Y,
        SketchPlane::YZ => DVec3::X,
    }
}

fn extrude_contour(
    contour: &[DVec2],
    plane: &SketchPlane,
    distance: f64,
    dir: DVec3,
    cap: bool,
) -> TriangleMesh {
    let n = contour.len();
    if n < 3 {
        return empty();
    }

    let mut verts: Vec<[f64; 3]> = Vec::new();
    let mut tris: Vec<[u32; 3]> = Vec::new();

    let normal_dir = dir.normalize();
    let ext_dir = normal_dir * distance;

    let bottom_start = 0u32;
    for p in contour {
        let v3 = project_to_3d(*p, plane);
        verts.push([v3.x, v3.y, v3.z]);
    }
    let top_start = n as u32;

    for p in contour {
        let v3 = project_to_3d(*p, plane) + ext_dir;
        verts.push([v3.x, v3.y, v3.z]);
    }

    if cap && n >= 3 {
        for i in 1..n - 1 {
            tris.push([bottom_start, bottom_start + i as u32 + 1, bottom_start + i as u32]);
        }
        for i in 1..n - 1 {
            tris.push([top_start, top_start + i as u32, top_start + i as u32 + 1]);
        }
    }

    for i in 0..n {
        let next = (i + 1) % n;
        let b0 = bottom_start + i as u32;
        let b1 = bottom_start + next as u32;
        let t0 = top_start + i as u32;
        let t1 = top_start + next as u32;
        tris.push([b0, b1, t1]);
        tris.push([b0, t1, t0]);
    }

    mesh(verts, tris)
}

pub fn extrude_sketch(sketch: &SketchDef, ext: &ExtrudeDef) -> TriangleMesh {
    let contours = tessellate_sketch(sketch);
    let dir = match ext.direction {
        Some(d) => DVec3::new(d[0], d[1], d[2]),
        None => extrude_direction(&sketch.plane),
    };
    merge_contours(contours, |contour| {
        extrude_contour(contour, &sketch.plane, ext.distance, dir, ext.cap)
    })
}

// ── Revolve ──────────────────────────────────────────────────────

fn revolve_contour(
    contour: &[DVec2],
    plane: &SketchPlane,
    angle: f64,
    segments: u32,
    axis: &RevolveAxis,
    cap: bool,
) -> TriangleMesh {
    let n = contour.len();
    if n < 3 || segments < 2 {
        return empty();
    }

    let seg = segments.max(3);
    let step = angle / seg as f64;
    let mut verts: Vec<[f64; 3]> = Vec::new();
    let mut tris: Vec<[u32; 3]> = Vec::new();

    for ring in 0..=seg {
        let theta = ring as f64 * step;
        let rot = match axis {
            RevolveAxis::X => DQuat::from_axis_angle(DVec3::X, theta),
            RevolveAxis::Y => DQuat::from_axis_angle(DVec3::Y, theta),
            RevolveAxis::Z => DQuat::from_axis_angle(DVec3::Z, theta),
        };

        for p in contour {
            let v3 = project_to_3d(*p, plane);
            let rotated = rot * v3;
            verts.push([rotated.x, rotated.y, rotated.z]);
        }
    }

    let pitch = n as u32;
    for ring in 0..seg {
        for i in 0..n {
            let next = (i + 1) % n;
            let r0 = ring * pitch + i as u32;
            let r1 = ring * pitch + next as u32;
            let r2 = (ring + 1) * pitch + i as u32;
            let r3 = (ring + 1) * pitch + next as u32;
            tris.push([r0, r2, r3]);
            tris.push([r0, r3, r1]);
        }
    }

    if cap && angle < std::f64::consts::TAU * 0.999 {
        let start_cap = 0u32;
        for i in 1..n - 1 {
            tris.push([start_cap, start_cap + i as u32, start_cap + i as u32 + 1]);
        }
        let end_cap = seg * pitch;
        for i in 1..n - 1 {
            tris.push([end_cap, end_cap + i as u32 + 1, end_cap + i as u32]);
        }
    }

    mesh(verts, tris)
}

pub fn revolve_sketch(sketch: &SketchDef, rev: &RevolveDef) -> TriangleMesh {
    let contours = tessellate_sketch(sketch);
    let seg = rev.segments.unwrap_or(32).max(3);
    merge_contours(contours, |contour| {
        revolve_contour(contour, &sketch.plane, rev.angle, seg, &rev.axis, rev.cap)
    })
}

// ── Sweep ────────────────────────────────────────────────────────

fn sweep_path_positions(path: &SweepPath, segments: u32) -> Vec<(DVec3, DQuat)> {
    let seg = segments.max(2) as usize;
    match path {
        SweepPath::Linear { direction, distance } => {
            let dir = DVec3::new(direction[0], direction[1], direction[2]).normalize();
            let up = if dir.y.abs() < 0.9 { DVec3::Y } else { DVec3::Z };
            let right = up.cross(dir).normalize();
            let real_up = dir.cross(right).normalize();
            let rot = DQuat::from_mat3(&DMat3::from_cols(right, real_up, dir));
            let step = distance / seg as f64;
            (0..=seg).map(|i| (dir * i as f64 * step, rot)).collect()
        }
        SweepPath::Arc { radius, angle, axis } => {
            let ax = DVec3::new(axis[0], axis[1], axis[2]).normalize();
            let step = angle / seg as f64;
            let up = if ax.y.abs() < 0.9 { DVec3::Y } else { DVec3::Z };
            let start_dir = ax.cross(up).normalize();
            let _real_up = start_dir.cross(ax).normalize();
            (0..=seg).map(|i| {
                let theta = i as f64 * step;
                let rot_seg = DQuat::from_axis_angle(ax, theta);
                let pos = rot_seg * (start_dir * radius);
                let right = pos.normalize();
                let tangent = ax.cross(right).normalize();
                let rot_up = tangent.cross(right).normalize();
                let frame_rot = DQuat::from_mat3(&DMat3::from_cols(
                    right, rot_up, tangent,
                ));
                (pos, frame_rot)
            }).collect()
        }
        SweepPath::Helix { radius, pitch, turns } => {
            let total_angle = turns * std::f64::consts::TAU;
            let step = total_angle / seg as f64;
            let height_step = pitch * turns / seg as f64;
            (0..=seg).map(|i| {
                let theta = i as f64 * step;
                let x = radius * theta.cos();
                let z = radius * theta.sin();
                let y = i as f64 * height_step;
                // Helix tangent: derivative of (r·cos θ, h·θ/(2π), r·sin θ)
                let dx_dt = -radius * theta.sin();
                let dy_dt = pitch / std::f64::consts::TAU;
                let dz_dt = radius * theta.cos();
                let tangent = DVec3::new(dx_dt, dy_dt, dz_dt).normalize();
                // Pick a stable "world up" that avoids degeneracy with the tangent
                let world_up = if tangent.dot(DVec3::Y).abs() < 0.95 {
                    DVec3::Y
                } else {
                    DVec3::Z
                };
                let right = world_up.cross(tangent).normalize();
                let frame_up = tangent.cross(right).normalize();
                // Columns: right (local X), frame_up (local Y), tangent (local Z)
                let rot = DQuat::from_mat3(&DMat3::from_cols(right, frame_up, tangent));
                (DVec3::new(x, y, z), rot)
            }).collect()
        }
    }
}

fn sweep_contour(
    contour: &[DVec2],
    _plane: &SketchPlane,          // plane is intentionally ignored here
    positions: &[(DVec3, DQuat)],
    cap: bool,
) -> TriangleMesh {
    let n = contour.len();
    if n < 3 || positions.len() < 2 {
        return empty();
    }

    // Use the 2D profile coordinates directly as (local_x, local_y, 0) offsets
    // inside the path frame.  Projecting through the sketch plane first would
    // put the profile into world space, so the subsequent frame rotation would
    // double-transform and collapse the cross-section into a flat ribbon.
    let profile_local: Vec<DVec3> = contour
        .iter()
        .map(|p| DVec3::new(p.x, p.y, 0.0))
        .collect();

    let mut verts: Vec<[f64; 3]> = Vec::new();
    let mut tris: Vec<[u32; 3]> = Vec::new();

    for (pos, rot) in positions {
        for p in &profile_local {
            // rotate the local profile point into the path frame, then translate
            let v = *rot * *p + *pos;
            verts.push([v.x, v.y, v.z]);
        }
    }

    let pitch = n as u32;
    for ring in 0..positions.len() - 1 {
        for i in 0..n {
            let next = (i + 1) % n;
            let r0 = ring as u32 * pitch + i as u32;
            let r1 = ring as u32 * pitch + next as u32;
            let r2 = (ring as u32 + 1) * pitch + i as u32;
            let r3 = (ring as u32 + 1) * pitch + next as u32;
            // Winding: outward normals pointing away from the tube axis
            tris.push([r0, r2, r3]);
            tris.push([r0, r3, r1]);
        }
    }

    if cap {
        let last = (positions.len() - 1) as u32 * pitch;
        for i in 1..n - 1 {
            // start cap: CCW when viewed from outside (looking down path)
            tris.push([0u32, i as u32, i as u32 + 1]);
            // end cap: CCW when viewed from outside (looking up path)
            tris.push([last, last + i as u32 + 1, last + i as u32]);
        }
    }

    mesh(verts, tris)
}

pub fn sweep_sketch(sketch: &SketchDef, swp: &SweepDef) -> TriangleMesh {
    let contours = tessellate_sketch(sketch);
    let seg = swp.segments.unwrap_or(32).max(2);
    let positions = sweep_path_positions(&swp.path, seg);
    merge_contours(contours, |contour| {
        sweep_contour(contour, &sketch.plane, &positions, swp.cap)
    })
}

// ── helpers ──────────────────────────────────────────────────────

fn empty() -> TriangleMesh {
    TriangleMesh { vertices: vec![], normals: vec![], uvs: vec![], triangles: vec![] }
}

fn mesh(vertices: Vec<[f64; 3]>, triangles: Vec<[u32; 3]>) -> TriangleMesh {
    TriangleMesh { vertices, normals: vec![], uvs: vec![], triangles }
}

fn merge_contours(
    contours: Vec<Vec<DVec2>>,
    f: impl Fn(&[DVec2]) -> TriangleMesh,
) -> TriangleMesh {
    let mut all_verts = Vec::new();
    let mut all_tris = Vec::new();
    for contour in &contours {
        let sub = f(contour);
        let base = all_verts.len() as u32;
        all_verts.extend(sub.vertices);
        for t in sub.triangles {
            all_tris.push([t[0] + base, t[1] + base, t[2] + base]);
        }
    }
    TriangleMesh { vertices: all_verts, normals: vec![], uvs: vec![], triangles: all_tris }
}
