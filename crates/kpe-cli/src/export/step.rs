use kpe_schema::geometry::TriangleMesh;

pub fn export_step(mesh: &TriangleMesh, output: &str) -> Result<(), String> {
    let mut s = String::new();
    s.push_str("ISO-10303-21;\nHEADER;\n");
    s.push_str("FILE_DESCRIPTION(('Tessellation'),'2;1');\n");
    s.push_str(&format!("FILE_NAME('{}','2025-01-01T12:00:00',(''),(''),'KPE','','');\n", output));
    s.push_str("FILE_SCHEMA(('TESS_TS'));\nENDSEC;\nDATA;\n");

    let mut id = 1u32;

    // CARTESIAN_POINT for each vertex
    for v in &mesh.vertices {
        s.push_str(&format!("#{}=CARTESIAN_POINT('',({},{},{}));\n", id, v[0], v[1], v[2]));
        id += 1;
    }
    let v_count = mesh.vertices.len() as u32;

    // single TRIANGULATED_FACE referencing all vertices + triangle index triplets
    // coordinates = (#1..#N), triangles = ((i1,i2,i3), (i4,i5,i6), ...)
    let refs: Vec<String> = (1..=v_count).map(|i| format!("#{}", i)).collect();
    let tri_indices: Vec<String> = mesh.triangles.iter()
        .map(|t| format!("({},{},{})", t[0] + 1, t[1] + 1, t[2] + 1))
        .collect();

    let face_ref = id;
    s.push_str(&format!(
        "#{}=TRIANGULATED_FACE('',({}),$,$,$,(\n{}));\n",
        face_ref,
        refs.join(","),
        tri_indices.join(",\n"),
    ));
    id += 1;

    // TESSELLATED_SHELL
    let shell_ref = id;
    s.push_str(&format!("#{}=TESSELLATED_SHELL('',(#{}));\n", shell_ref, face_ref));
    id += 1;

    // context
    let ctx_ref = id;
    s.push_str(&format!(
        "#{}=(GEOMETRIC_REPRESENTATION_CONTEXT(3)GLOBAL_UNIT_ASSIGNED_CONTEXT((#{})));\n",
        ctx_ref, id + 1
    ));
    id += 1;

    // units
    s.push_str(&format!("#{}=SI_UNIT($,.MILLI.)LENGTH_UNIT();\n", id)); id += 1;

    // MANIFOLD_SURFACE_SHAPE_REPRESENTATION
    s.push_str(&format!(
        "#{}=MANIFOLD_SURFACE_SHAPE_REPRESENTATION('',(#{}),#{});\n",
        id, shell_ref, ctx_ref
    ));

    s.push_str("ENDSEC;\nEND-ISO-10303-21;\n");

    std::fs::write(output, &s).map_err(|e| e.to_string())
}
