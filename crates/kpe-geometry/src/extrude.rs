use glam::{DVec2, DVec3};
use kpe_schema::geometry::{ExtrudeDef, SketchDef, SketchPlane, TriangleMesh};
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
        return TriangleMesh {
            vertices: vec![],
            normals: vec![],
            uvs: vec![],
            triangles: vec![],
        };
    }

    let mut verts: Vec<[f64; 3]> = Vec::new();
    let mut tris: Vec<[u32; 3]> = Vec::new();

    let normal_dir = dir.normalize();
    let ext_dir = normal_dir * distance;

    // bottom vertices
    let bottom_start = 0u32;
    for p in contour {
        let v3 = project_to_3d(*p, plane);
        verts.push([v3.x, v3.y, v3.z]);
    }
    let bottom_end = n as u32;

    // top vertices
    let top_start = bottom_end;
    for p in contour {
        let v3 = project_to_3d(*p, plane) + ext_dir;
        verts.push([v3.x, v3.y, v3.z]);
    }

    // bottom cap (fan from first vertex, reversed normal for outward)
    if cap && n >= 3 {
        for i in 1..n - 1 {
            tris.push([bottom_start, bottom_start + i as u32 + 1, bottom_start + i as u32]);
        }
    }

    // top cap (fan from first vertex)
    if cap && n >= 3 {
        for i in 1..n - 1 {
            tris.push([top_start, top_start + i as u32, top_start + i as u32 + 1]);
        }
    }

    // side walls (quad strip)
    for i in 0..n {
        let next = (i + 1) % n;
        let b0 = bottom_start + i as u32;
        let b1 = bottom_start + next as u32;
        let t0 = top_start + i as u32;
        let t1 = top_start + next as u32;
        tris.push([b0, b1, t1]);
        tris.push([b0, t1, t0]);
    }

    TriangleMesh {
        vertices: verts,
        normals: vec![],
        uvs: vec![],
        triangles: tris,
    }
}

pub fn extrude_sketch(
    sketch: &SketchDef,
    ext: &ExtrudeDef,
) -> TriangleMesh {
    let contours = tessellate_sketch(sketch);
    let dir = match ext.direction {
        Some(d) => DVec3::new(d[0], d[1], d[2]),
        None => extrude_direction(&sketch.plane),
    };

    let mut all_verts = Vec::new();
    let mut all_tris = Vec::new();

    for contour in &contours {
        let sub = extrude_contour(contour, &sketch.plane, ext.distance, dir, ext.cap);
        let base = all_verts.len() as u32;
        all_verts.extend(sub.vertices);
        for t in sub.triangles {
            all_tris.push([t[0] + base, t[1] + base, t[2] + base]);
        }
    }

    TriangleMesh {
        vertices: all_verts,
        normals: vec![],
        uvs: vec![],
        triangles: all_tris,
    }
}
