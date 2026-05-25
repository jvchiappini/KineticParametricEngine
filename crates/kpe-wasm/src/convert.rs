use kpe_schema::geometry::TriangleMesh;
use kpe_schema::recipe::KPERecipe;

pub fn recipe_to_json(recipe: &KPERecipe) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(recipe)
}

pub fn json_to_recipe(json: &str) -> Result<KPERecipe, serde_json::Error> {
    serde_json::from_str(json)
}

pub fn mesh_to_json(mesh: &TriangleMesh) -> Result<String, serde_json::Error> {
    serde_json::to_string(mesh)
}

pub fn mesh_vertices_f32(mesh: &TriangleMesh) -> Vec<f32> {
    let mut flat = Vec::with_capacity(mesh.vertices.len() * 3);
    for v in &mesh.vertices {
        flat.push(v[0] as f32);
        flat.push(v[1] as f32);
        flat.push(v[2] as f32);
    }
    flat
}

pub fn mesh_triangles_u32(mesh: &TriangleMesh) -> Vec<u32> {
    let mut flat = Vec::with_capacity(mesh.triangles.len() * 3);
    for tri in &mesh.triangles {
        flat.push(tri[0]);
        flat.push(tri[1]);
        flat.push(tri[2]);
    }
    flat
}
