use std::path::PathBuf;
use rfd::FileDialog;
use crate::commands::Command;
use crate::document::Document;

pub fn save_dialog(_doc: &Document) -> Option<PathBuf> {
    let path = FileDialog::new()
        .add_filter("KPE Document", &["kpe"])
        .set_file_name("untitled.kpe")
        .save_file();
    path
}

pub fn open_dialog() -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("KPE Document", &["kpe"])
        .pick_file()
}

pub fn export_stl_dialog() -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("STL Binary", &["stl"])
        .set_file_name("model.stl")
        .save_file()
}

pub fn export_obj_dialog() -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("OBJ Wavefront", &["obj"])
        .set_file_name("model.obj")
        .save_file()
}

pub fn save_to_file(path: &std::path::Path, doc: &Document) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&doc.recipe)
        .map_err(|e| format!("serialization error: {}", e))?;
    std::fs::write(path, json)
        .map_err(|e| format!("write error: {}", e))?;
    Ok(())
}

pub fn load_from_file(path: &std::path::Path) -> Result<Document, String> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| format!("read error: {}", e))?;
    let recipe: kpe_schema::recipe::KPERecipe = serde_json::from_str(&data)
        .map_err(|e| format!("deserialization error: {}", e))?;
    let mut doc = Document::new();
    doc.recipe = recipe;
    doc.evaluate_all();
    doc.selection = doc.all_node_ids().first().cloned();
    Ok(doc)
}

pub fn export_stl(path: &std::path::Path, doc: &Document) -> Result<(), String> {
    use std::io::Write;

    let mut buf: Vec<u8> = Vec::new();

    let total_tris: usize = doc.evaluated.meshes.values()
        .map(|m| m.triangles.len())
        .sum();

    if total_tris > 0xFFFF_FFFF {
        return Err("too many triangles for binary STL".into());
    }

    // STL header (80 bytes, usually ignored)
    let header = format!("KPE Export\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0");
    buf.write_all(header.as_bytes()).map_err(|e| e.to_string())?;

    // Number of triangles (u32 LE)
    let tri_count = total_tris as u32;
    buf.write_all(&tri_count.to_le_bytes()).map_err(|e| e.to_string())?;

    for mesh in doc.evaluated.meshes.values() {
        for tri in &mesh.triangles {
            let v0 = get_vertex(&mesh.vertices, tri[0]);
            let v1 = get_vertex(&mesh.vertices, tri[1]);
            let v2 = get_vertex(&mesh.vertices, tri[2]);

            // Compute face normal
            let (nx, ny, nz) = compute_normal(v0, v1, v2);

            buf.write_all(&nx.to_le_bytes()).map_err(|e| e.to_string())?;
            buf.write_all(&ny.to_le_bytes()).map_err(|e| e.to_string())?;
            buf.write_all(&nz.to_le_bytes()).map_err(|e| e.to_string())?;

            buf.write_all(&v0.0.to_le_bytes()).map_err(|e| e.to_string())?;
            buf.write_all(&v0.1.to_le_bytes()).map_err(|e| e.to_string())?;
            buf.write_all(&v0.2.to_le_bytes()).map_err(|e| e.to_string())?;

            buf.write_all(&v1.0.to_le_bytes()).map_err(|e| e.to_string())?;
            buf.write_all(&v1.1.to_le_bytes()).map_err(|e| e.to_string())?;
            buf.write_all(&v1.2.to_le_bytes()).map_err(|e| e.to_string())?;

            buf.write_all(&v2.0.to_le_bytes()).map_err(|e| e.to_string())?;
            buf.write_all(&v2.1.to_le_bytes()).map_err(|e| e.to_string())?;
            buf.write_all(&v2.2.to_le_bytes()).map_err(|e| e.to_string())?;

            // Attribute byte count (u16 LE) - usually 0
            buf.write_all(&0u16.to_le_bytes()).map_err(|e| e.to_string())?;
        }
    }

    std::fs::write(path, buf).map_err(|e| format!("write error: {}", e))
}

pub fn export_obj(path: &std::path::Path, doc: &Document) -> Result<(), String> {
    use std::io::Write;

    let mut content = String::new();
    content.push_str("# KPE Export\n");
    content.push_str(&format!("# Triangles: {}\n", doc.evaluated.triangle_count()));

    let mut vert_offset: u32 = 1;

    for (id, mesh) in &doc.evaluated.meshes {
        content.push_str(&format!("o {}\n", id));

        for v in &mesh.vertices {
            content.push_str(&format!("v {} {} {}\n", v[0], v[1], v[2]));
        }

        for n in &mesh.normals {
            content.push_str(&format!("vn {} {} {}\n", n[0], n[1], n[2]));
        }

        for tri in &mesh.triangles {
            let a = tri[0] + vert_offset;
            let b = tri[1] + vert_offset;
            let c = tri[2] + vert_offset;
            content.push_str(&format!("f {} {} {}\n", a, b, c));
        }

        vert_offset += mesh.vertices.len() as u32;
    }

    let mut file = std::fs::File::create(path)
        .map_err(|e| format!("create error: {}", e))?;
    file.write_all(content.as_bytes())
        .map_err(|e| format!("write error: {}", e))?;

    Ok(())
}

fn get_vertex(verts: &[[f64; 3]], idx: u32) -> (f32, f32, f32) {
    let v = &verts[idx as usize];
    (v[0] as f32, v[1] as f32, v[2] as f32)
}

fn compute_normal(a: (f32, f32, f32), b: (f32, f32, f32), c: (f32, f32, f32)) -> (f32, f32, f32) {
    let ux = b.0 - a.0;
    let uy = b.1 - a.1;
    let uz = b.2 - a.2;
    let vx = c.0 - a.0;
    let vy = c.1 - a.1;
    let vz = c.2 - a.2;

    let nx = uy * vz - uz * vy;
    let ny = uz * vx - ux * vz;
    let nz = ux * vy - uy * vx;

    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    if len > 1e-10 {
        (nx / len, ny / len, nz / len)
    } else {
        (0.0, 1.0, 0.0)
    }
}

pub fn open_document() -> Option<Document> {
    let path = open_dialog()?;
    load_from_file(&path).ok()
}

pub struct SaveCommand;

impl Command for SaveCommand {
    fn execute(&mut self, doc: &mut Document) {
        let path = save_dialog(doc);
        if let Some(p) = path {
            let _ = save_to_file(&p, doc);
        }
    }

    fn undo(&mut self, _doc: &mut Document) {}

    fn description(&self) -> &str {
        "Save Document"
    }
}
