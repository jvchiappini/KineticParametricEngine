use glam::DVec3;
use kpe_schema::geometry::TriangleMesh;
use std::collections::{HashMap, HashSet, VecDeque};

/// Tolerance for considering two triangle normals as coplanar.
/// cos(0.5°) ≈ 0.99996 — very tight; only truly coplanar triangles merge.
const COPLANAR_COS_TOLERANCE: f64 = 0.9999;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A face is a set of coplanar, edge-connected triangles with a closed
/// boundary loop.  Faces are the atomic unit of SketchUp-style push‑pull
/// extrusion.
#[derive(Debug, Clone)]
pub struct Face {
    /// Indices into the parent mesh's `triangles` array.
    pub triangle_indices: Vec<usize>,

    /// Ordered boundary edges `(v_start, v_end)` forming a closed loop.
    /// The loop winds counter‑clockwise when viewed from the face normal
    /// direction (right‑hand rule around `normal`).
    pub boundary_loop: Vec<(u32, u32)>,

    /// Unit face normal (average of all triangle normals in the face).
    pub normal: DVec3,

    /// Geometric centre of the face (average of all face vertices).
    pub center: DVec3,
}

// ---------------------------------------------------------------------------
// Face detection
// ---------------------------------------------------------------------------

/// Detect all faces in a triangle mesh.
///
/// A face is a maximal set of triangles that are:
/// 1. **edge‑connected** — every triangle shares at least one edge with
///    another triangle in the set;
/// 2. **coplanar** — all triangle normals point within `COPLANAR_COS_TOLERANCE`
///    of each other (≈ 0.8°).
///
/// Each face comes with its closed boundary loop, oriented consistently.
pub fn detect_faces(mesh: &TriangleMesh) -> Vec<Face> {
    if mesh.triangles.is_empty() {
        return vec![];
    }

    // 1. Per‑triangle normals and edge-adjacency map.
    let tri_normals: Vec<DVec3> = mesh
        .triangles
        .iter()
        .map(|tri| triangle_normal(&mesh.vertices, tri))
        .collect();

    let edge_map = build_edge_adjacency(&mesh.triangles);

    // 2. Flood‑fill faces.
    let mut assigned: Vec<bool> = vec![false; mesh.triangles.len()];
    let mut faces: Vec<Face> = Vec::new();

    for seed in 0..mesh.triangles.len() {
        if assigned[seed] {
            continue;
        }

        let seed_normal = tri_normals[seed];
        let mut face_tris: Vec<usize> = Vec::new();
        let mut queue: VecDeque<usize> = VecDeque::new();
        queue.push_back(seed);
        assigned[seed] = true;

        while let Some(ti) = queue.pop_front() {
            face_tris.push(ti);
            let tri = &mesh.triangles[ti];
            for e in triangle_edges(tri) {
                if let Some(neighbours) = edge_map.get(&e) {
                    for &adj_ti in neighbours {
                        if !assigned[adj_ti]
                            && tri_normals[adj_ti].dot(seed_normal) > COPLANAR_COS_TOLERANCE
                        {
                            assigned[adj_ti] = true;
                            queue.push_back(adj_ti);
                        }
                    }
                }
            }
        }

        // 3. Extract boundary loop.
        let face_set: HashSet<usize> = face_tris.iter().copied().collect();
        let boundary = extract_boundary_loop(&face_tris, &mesh.triangles, &edge_map, &face_set);

        // 4. Compute average normal and centre.
        let avg_normal = face_tris
            .iter()
            .map(|&ti| tri_normals[ti])
            .sum::<DVec3>()
            / face_tris.len() as f64;
        let avg_normal = avg_normal.normalize();

        let center = face_center(&mesh.vertices, &face_tris, &mesh.triangles);

        faces.push(Face {
            triangle_indices: face_tris,
            boundary_loop: boundary,
            normal: avg_normal,
            center,
        });
    }

    faces
}

// ---------------------------------------------------------------------------
// Extrusion
// ---------------------------------------------------------------------------

/// Extrude a face along its normal by `distance`.
///
/// The original face triangles are replaced with:
///   - A **top cap** (original face translated by `normal × distance`).
///   - **Side quads** (two triangles per boundary edge) connecting the
///     original boundary to the top cap.
///
/// All other triangles in the mesh are preserved unchanged.
pub fn extrude_face(mesh: &TriangleMesh, face: &Face, distance: f64) -> TriangleMesh {
    if distance.abs() < 1e-12 || face.boundary_loop.is_empty() {
        return mesh.clone();
    }

    let dir = face.normal * distance;

    // 1. Build the set of *all* vertex indices used by face triangles.
    let face_vert_set: HashSet<u32> = face
        .triangle_indices
        .iter()
        .flat_map(|&ti| {
            let tri = &mesh.triangles[ti];
            [tri[0], tri[1], tri[2]]
        })
        .collect();
    let face_verts: Vec<u32> = face_vert_set.into_iter().collect();
    let base = mesh.vertices.len() as u32;

    // Map original vertex index → new top‑cap vertex index.
    let mut vert_map: HashMap<u32, u32> = HashMap::new();
    for (i, &v) in face_verts.iter().enumerate() {
        vert_map.insert(v, base + i as u32);
    }

    // 2. Build the vertex list: original + top‑cap.
    let mut new_verts = mesh.vertices.clone();
    for &v in &face_verts {
        let orig = DVec3::from_slice(&mesh.vertices[v as usize]);
        let top = orig + dir;
        new_verts.push([top.x, top.y, top.z]);
    }

    // 3. Copy over *non‑face* triangles.
    let face_set: HashSet<usize> = face.triangle_indices.iter().copied().collect();
    let mut new_tris: Vec<[u32; 3]> = Vec::with_capacity(
        mesh.triangles.len() - face.triangle_indices.len()
            + face.boundary_loop.len() * 2
            + face.triangle_indices.len(),
    );
    for (i, tri) in mesh.triangles.iter().enumerate() {
        if !face_set.contains(&i) {
            new_tris.push(*tri);
        }
    }

    // 4. Side quads from boundary edges.
    //    Each boundary edge (a,b) produces a quad (a, b, b_top, a_top)
    //    split into triangles (a, b, b_top) and (a, b_top, a_top).
    //    Windings are chosen so the side normal points outward.
    for &(a, b) in &face.boundary_loop {
        let a_top = vert_map[&a];
        let b_top = vert_map[&b];
        new_tris.push([a, b, b_top]);
        new_tris.push([a, b_top, a_top]);
    }

    // 5. Top cap: same topology as the original face, translated vertices.
    for &ti in &face.triangle_indices {
        let tri = mesh.triangles[ti];
        new_tris.push([vert_map[&tri[0]], vert_map[&tri[1]], vert_map[&tri[2]]]);
    }

    TriangleMesh {
        vertices: new_verts,
        normals: vec![],
        uvs: vec![],
        triangles: new_tris,
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Edge key: unordered pair `(min, max)`.
type EdgeKey = (u32, u32);

fn edge_key(a: u32, b: u32) -> EdgeKey {
    (a.min(b), a.max(b))
}

fn triangle_edges(tri: &[u32; 3]) -> [EdgeKey; 3] {
    [
        edge_key(tri[0], tri[1]),
        edge_key(tri[1], tri[2]),
        edge_key(tri[2], tri[0]),
    ]
}

fn triangle_normal(verts: &[[f64; 3]], tri: &[u32; 3]) -> DVec3 {
    let a = DVec3::from_slice(&verts[tri[0] as usize]);
    let b = DVec3::from_slice(&verts[tri[1] as usize]);
    let c = DVec3::from_slice(&verts[tri[2] as usize]);
    (b - a).cross(c - a).normalize()
}

/// Build a map from edge → list of triangle indices sharing that edge.
fn build_edge_adjacency(triangles: &[[u32; 3]]) -> HashMap<EdgeKey, Vec<usize>> {
    let mut map: HashMap<EdgeKey, Vec<usize>> = HashMap::new();
    for (i, tri) in triangles.iter().enumerate() {
        for e in triangle_edges(tri) {
            map.entry(e).or_default().push(i);
        }
    }
    map
}

/// Extract the oriented boundary loop for a face.
///
/// A boundary edge is an edge that belongs to **exactly one** triangle
/// in the face (edges shared by two face triangles are interior).
///
/// Returns edges ordered into a closed counter‑clockwise loop (when
/// viewed from the face normal direction, assuming the normal points
/// outward / upward).
fn extract_boundary_loop(
    face_tris: &[usize],
    all_tris: &[[u32; 3]],
    edge_map: &HashMap<EdgeKey, Vec<usize>>,
    face_set: &HashSet<usize>,
) -> Vec<(u32, u32)> {
    // Collect all boundary edges: edges with exactly one face triangle.
    let mut boundary_set: Vec<(u32, u32)> = Vec::new();

    for &ti in face_tris {
        let tri = &all_tris[ti];
        for e in triangle_edges(tri) {
            if let Some(adj) = edge_map.get(&e) {
                let face_adj_count = adj.iter().filter(|&&a| face_set.contains(&a)).count();
                if face_adj_count == 1 {
                    // Determine the oriented edge direction as it appears in *our* triangle.
                    // This ensures consistent winding across all boundary edges.
                    let oriented = if tri[0] == e.0 || tri[1] == e.0 || tri[2] == e.0 {
                        if edge_contains_ordered(tri, e.0, e.1) {
                            (e.0, e.1)
                        } else {
                            (e.1, e.0)
                        }
                    } else {
                        (e.1, e.0)
                    };
                    boundary_set.push(oriented);
                }
            }
        }
    }

    // Sort boundary edges into a closed loop.
    order_boundary_loop(&boundary_set)
}

/// Check if an oriented edge `(from, to)` appears in triangle `tri`.
fn edge_contains_ordered(tri: &[u32; 3], from: u32, to: u32) -> bool {
    (tri[0] == from && tri[1] == to)
        || (tri[1] == from && tri[2] == to)
        || (tri[2] == from && tri[0] == to)
}

/// Order a set of boundary edges into a closed loop.
///
/// The input edges must already be oriented consistently.  We walk from
/// edge to edge by matching `(a, b) → (b, c) → (c, d) → …` until
/// returning to the start vertex.
fn order_boundary_loop(edges: &[(u32, u32)]) -> Vec<(u32, u32)> {
    if edges.is_empty() {
        return vec![];
    }

    // Build forward map: start_vertex → end_vertex.
    let mut forward: HashMap<u32, u32> = HashMap::with_capacity(edges.len());
    for &(a, b) in edges {
        forward.insert(a, b);
    }

    let mut loop_edges: Vec<(u32, u32)> = Vec::with_capacity(edges.len());
    let start = edges[0].0;
    let mut current = start;

    for _ in 0..edges.len() {
        if let Some(&next) = forward.get(&current) {
            loop_edges.push((current, next));
            current = next;
            if current == start {
                break;
            }
        } else {
            // Broken chain — fall back to returning whatever we have.
            break;
        }
    }

    loop_edges
}

/// Compute the average centre of all vertices belonging to a set of triangles.
fn face_center(
    verts: &[[f64; 3]],
    tri_indices: &[usize],
    all_tris: &[[u32; 3]],
) -> DVec3 {
    let mut sum = DVec3::ZERO;
    let mut count = 0u32;
    let mut seen: HashSet<u32> = HashSet::new();
    for &ti in tri_indices {
        for &v in &all_tris[ti] {
            if seen.insert(v) {
                let p = DVec3::from_slice(&verts[v as usize]);
                sum += p;
                count += 1;
            }
        }
    }
    if count == 0 {
        return DVec3::ZERO;
    }
    sum / count as f64
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a single‑quad mesh (two triangles) on the XY plane.
    fn xy_quad_mesh() -> TriangleMesh {
        TriangleMesh {
            vertices: vec![
                [0.0, 0.0, 0.0], // 0
                [2.0, 0.0, 0.0], // 1
                [2.0, 2.0, 0.0], // 2
                [0.0, 2.0, 0.0], // 3
            ],
            normals: vec![],
            uvs: vec![],
            triangles: vec![[0, 1, 2], [0, 2, 3]],
        }
    }

    /// Helper: a box mesh (6 faces, 12 triangles).
    fn unit_box_mesh() -> TriangleMesh {
        // Corners of a unit cube at origin.
        let (x0, x1, y0, y1, z0, z1) = (0.0, 1.0, 0.0, 1.0, 0.0, 1.0);
        let v: Vec<[f64; 3]> = vec![
            [x0, y0, z0], // 0
            [x1, y0, z0], // 1
            [x1, y1, z0], // 2
            [x0, y1, z0], // 3
            [x0, y0, z1], // 4
            [x1, y0, z1], // 5
            [x1, y1, z1], // 6
            [x0, y1, z1], // 7
        ];
        // 12 triangles forming 6 faces.
        let t: Vec<[u32; 3]> = vec![
            // bottom (y=0, normal -Y)
            [0, 2, 1], [0, 3, 2],
            // top (y=1, normal +Y)
            [4, 5, 6], [4, 6, 7],
            // front (z=0, normal -Z)
            [0, 1, 5], [0, 5, 4],
            // back (z=1, normal +Z)
            [3, 7, 6], [3, 6, 2],
            // left (x=0, normal -X)
            [0, 4, 7], [0, 7, 3],
            // right (x=1, normal +X)
            [1, 2, 6], [1, 6, 5],
        ];
        TriangleMesh { vertices: v, normals: vec![], uvs: vec![], triangles: t }
    }

    // ── detect_faces tests ───────────────────────────────────────────

    #[test]
    fn test_detect_faces_single_quad() {
        let mesh = xy_quad_mesh();
        let faces = detect_faces(&mesh);
        // Both triangles are coplanar and edge‑connected → one face.
        assert_eq!(faces.len(), 1, "Quad should produce exactly one face");
        let f = &faces[0];
        assert_eq!(f.triangle_indices.len(), 2);
        assert!(
            (f.normal - DVec3::Z).length() < 1e-6,
            "Expected normal +Z, got {:?}",
            f.normal,
        );
        // Boundary loop should be the outer square: 4 edges.
        assert_eq!(f.boundary_loop.len(), 4, "Quad boundary should have 4 edges");
    }

    #[test]
    fn test_detect_faces_unit_box() {
        let mesh = unit_box_mesh();
        let faces = detect_faces(&mesh);
        // A box has 6 faces.
        assert_eq!(faces.len(), 6, "Unit box should have exactly 6 faces");
        // Each face should have 2 triangles and a 4‑edge boundary loop.
        for f in &faces {
            assert_eq!(f.triangle_indices.len(), 2);
            assert_eq!(f.boundary_loop.len(), 4);
            // Every normal should be axis‑aligned.
            let n = f.normal;
            let axis_aligned = n.x.abs() > 0.99
                || n.y.abs() > 0.99
                || n.z.abs() > 0.99;
            assert!(axis_aligned, "Face normal {:?} is not axis‑aligned", n);
        }
    }

    #[test]
    fn test_detect_faces_empty_mesh() {
        let mesh = TriangleMesh {
            vertices: vec![],
            normals: vec![],
            uvs: vec![],
            triangles: vec![],
        };
        assert!(detect_faces(&mesh).is_empty());
    }

    // ── extrude_face tests ───────────────────────────────────────────

    #[test]
    fn test_extrude_face_quad() {
        let mesh = xy_quad_mesh();
        let faces = detect_faces(&mesh);
        assert_eq!(faces.len(), 1, "Quad should be one face");

        let result = extrude_face(&mesh, &faces[0], 2.0);

        // Original: 4 verts, 2 tris.
        // Extrude: 4 new top verts. Triangles: 0 face tris removed,
        //   4 boundary edges → 8 side tris, 2 top tris = 10 tris.
        assert_eq!(
            result.vertices.len(),
            8,
            "Expected 4 original + 4 top vertices, got {}",
            result.vertices.len(),
        );
        assert_eq!(
            result.triangles.len(),
            10,
            "Expected 8 side + 2 top triangles, got {}",
            result.triangles.len(),
        );

        // Spot‑check: top‑cap vertices should be at z = 2.0.
        let top_verts: Vec<[f64; 3]> = result.vertices[4..].to_vec();
        for v in &top_verts {
            assert!(
                (v[2] - 2.0).abs() < 1e-10,
                "Top vertex {:?} should have z=2.0",
                v,
            );
        }
    }

    #[test]
    fn test_extrude_face_box_face() {
        let mesh = unit_box_mesh();
        let faces = detect_faces(&mesh);
        assert_eq!(faces.len(), 6);

        // Extrude the top face (normal +Y).
        let top_face = faces
            .iter()
            .find(|f| f.normal.y > 0.9)
            .expect("Should find a face with normal +Y");

        let result = extrude_face(&mesh, top_face, 1.5);

        // Original: 8 verts, 12 tris.
        // Top face: 2 tris removed, 4 boundary edges → 8 side tris, 2 top tris
        // Remaining: 10 original tris + 8 side + 2 top = 20 tris
        // Vertices: 8 original + 4 top = 12
        assert_eq!(result.vertices.len(), 12);
        assert_eq!(result.triangles.len(), 20);

        // Top‑cap vertices should be at y = 1.0 + 1.5 = 2.5.
        for v in &result.vertices {
            // Vertices that are part of the top cap (y ≈ 1.0 originally) become y ≈ 2.5
            if (v[1] - 1.0).abs() < 1e-6 {
                // This was a top‑face vertex in the original — in the extruded
                // mesh it should still exist at the original position (it's
                // now a *bottom* vertex of the extrusion column).
            }
        }
        // The new top vertices (indices 8–11) should be at y ≈ 2.5.
        for v in &result.vertices[8..] {
            assert!(
                (v[1] - 2.5).abs() < 1e-10,
                "New top vertex {:?} should have y = 2.5",
                v,
            );
        }
    }

    #[test]
    fn test_extrude_face_zero_distance() {
        let mesh = xy_quad_mesh();
        let faces = detect_faces(&mesh);
        let result = extrude_face(&mesh, &faces[0], 0.0);
        // Zero distance → unchanged.
        assert_eq!(result.vertices.len(), mesh.vertices.len());
        assert_eq!(result.triangles.len(), mesh.triangles.len());
    }

    #[test]
    fn test_extrude_face_negative_distance() {
        let mesh = xy_quad_mesh();
        let faces = detect_faces(&mesh);
        let result = extrude_face(&mesh, &faces[0], -1.0);
        // Negative extrusion should push the face downward.
        let top_verts: Vec<[f64; 3]> = result.vertices[4..].to_vec();
        for v in &top_verts {
            assert!(
                (v[2] - (-1.0)).abs() < 1e-10,
                "Top vertex {:?} should have z = -1.0",
                v,
            );
        }
    }

    #[test]
    fn test_extrude_face_manifold_edge_count() {
        // Verify that every interior edge is shared by exactly two triangles
        // (manifold condition).
        let mesh = xy_quad_mesh();
        let faces = detect_faces(&mesh);
        let result = extrude_face(&mesh, &faces[0], 2.0);

        let edge_count = build_edge_adjacency(&result.triangles);
        for (_edge, tris) in &edge_count {
            assert!(
                tris.len() == 1 || tris.len() == 2,
                "Edge {:?} is shared by {} triangles (non‑manifold)",
                _edge,
                tris.len(),
            );
        }
    }
}
