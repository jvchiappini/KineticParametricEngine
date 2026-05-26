use glam::DVec3;
use kpe_schema::geometry::TriangleMesh;

/// Extrude a face of a mesh along its normal.
///
/// Takes a `TriangleMesh`, a face index (triangle index), and an extrusion distance.
/// Returns a new mesh with the face extruded. Uses the face normal to determine
/// direction.
pub fn extrude_face(mesh: &TriangleMesh, face_idx: usize, distance: f64) -> TriangleMesh {
    if distance == 0.0 || face_idx >= mesh.triangles.len() {
        return mesh.clone();
    }

    let tri = mesh.triangles[face_idx];
    let a = vec3(&mesh.vertices, tri[0]);
    let b = vec3(&mesh.vertices, tri[1]);
    let c = vec3(&mesh.vertices, tri[2]);

    let normal = (b - a).cross(c - a).normalize();
    let dir = normal * distance;

    let mut verts = mesh.vertices.clone();
    let mut tris: Vec<[u32; 3]> = Vec::new();

    let base = verts.len() as u32;
    for &idx in &tri {
        let v = vec3(&mesh.vertices, idx);
        verts.push([v.x + dir.x, v.y + dir.y, v.z + dir.z]);
    }

    tris.push([tri[0], tri[1], base + 1]);
    tris.push([tri[0], base + 1, base]);
    tris.push([tri[1], tri[2], base + 2]);
    tris.push([tri[1], base + 2, base + 1]);
    tris.push([tri[2], tri[0], base]);
    tris.push([tri[2], base, base + 2]);

    tris.push([base, base + 1, base + 2]);

    for (i, t) in mesh.triangles.iter().enumerate() {
        if i != face_idx {
            tris.push(*t);
        }
    }

    TriangleMesh {
        vertices: verts,
        normals: vec![],
        uvs: vec![],
        triangles: tris,
    }
}

fn vec3(verts: &[[f64; 3]], idx: u32) -> DVec3 {
    let v = verts[idx as usize];
    DVec3::new(v[0], v[1], v[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_single_tri() -> TriangleMesh {
        TriangleMesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![],
            uvs: vec![],
            triangles: vec![[0, 1, 2]],
        }
    }

    #[test]
    fn test_extrude_face_zero_distance() {
        let mesh = make_single_tri();
        let result = extrude_face(&mesh, 0, 0.0);
        assert_eq!(result.vertices.len(), 3);
    }

    #[test]
    fn test_extrude_face_adds_vertices() {
        let mesh = make_single_tri();
        let result = extrude_face(&mesh, 0, 2.0);
        assert_eq!(result.vertices.len(), 6);
        assert_eq!(result.triangles.len(), 7);
    }

    #[test]
    fn test_extrude_face_normal_direction() {
        let mesh = make_single_tri();
        let result = extrude_face(&mesh, 0, 2.0);
        let top = result.vertices[5];
        assert!((top[2] - 2.0).abs() < 1e-10);
    }
}
