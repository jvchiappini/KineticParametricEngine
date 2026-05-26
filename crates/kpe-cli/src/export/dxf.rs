use kpe_schema::geometry::TriangleMesh;

fn min_max(v: &[[f64; 3]]) -> ([f64; 3], [f64; 3]) {
    let mut lo = [f64::MAX, f64::MAX, f64::MAX];
    let mut hi = [f64::MIN, f64::MIN, f64::MIN];
    for p in v {
        for i in 0..3 {
            if p[i] < lo[i] { lo[i] = p[i]; }
            if p[i] > hi[i] { hi[i] = p[i]; }
        }
    }
    (lo, hi)
}

pub fn export_dxf(mesh: &TriangleMesh, output: &str) -> Result<(), String> {
    let mut s = String::new();

    // header
    let (extmin, extmax) = min_max(&mesh.vertices);
    s.push_str("  0\nSECTION\n  2\nHEADER\n");
    s.push_str("  9\n$ACADVER\n  1\nAC1009\n");
    s.push_str("  9\n$INSBASE\n 10\n0.0\n 20\n0.0\n 30\n0.0\n");
    s.push_str(&format!("  9\n$EXTMIN\n 10\n{}\n 20\n{}\n 30\n{}\n", extmin[0], extmin[1], extmin[2]));
    s.push_str(&format!("  9\n$EXTMAX\n 10\n{}\n 20\n{}\n 30\n{}\n", extmax[0], extmax[1], extmax[2]));
    s.push_str("  0\nENDSEC\n");

    // entities
    s.push_str("  0\nSECTION\n  2\nENTITIES\n");
    for tri in &mesh.triangles {
        let a = &mesh.vertices[tri[0] as usize];
        let b = &mesh.vertices[tri[1] as usize];
        let c = &mesh.vertices[tri[2] as usize];
        s.push_str(&format!(
            "  0\n3DFACE\n  8\n0\n\
             10\n{}\n 20\n{}\n 30\n{}\n\
             11\n{}\n 21\n{}\n 31\n{}\n\
             12\n{}\n 22\n{}\n 32\n{}\n\
             13\n{}\n 23\n{}\n 33\n{}\n",
            a[0], a[1], a[2],
            b[0], b[1], b[2],
            c[0], c[1], c[2],
            c[0], c[1], c[2],
        ));
    }
    s.push_str("  0\nENDSEC\n  0\nEOF\n");

    std::fs::write(output, &s).map_err(|e| e.to_string())
}
