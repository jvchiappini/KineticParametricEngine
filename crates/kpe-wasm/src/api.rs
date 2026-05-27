use wasm_bindgen::prelude::*;
use kpe_schema::recipe::KPERecipe;
use kpe_schema::geometry::TriangleMesh;
use kpe_parametric::Solver as ParamSolver;
use kpe_geometry::csg::CsgKernel;
use kpe_geometry::sketch::SketchDocument;

#[wasm_bindgen]
pub fn hello() -> String {
    "KPE Engine v0.1 — constraint solver ready".to_string()
}

// ── Sketch Document ──────────────────────────────────────────────

#[wasm_bindgen]
pub fn sketch_new() -> String {
    let doc = SketchDocument::new();
    serde_json::to_string(&doc).unwrap_or_default()
}

#[wasm_bindgen]
pub fn sketch_from_json(json: &str) -> Result<String, JsValue> {
    let doc: SketchDocument = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {e}")))?;
    serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {e}")))
}

#[wasm_bindgen]
pub fn sketch_add_line(json: &str, x1: f64, y1: f64, x2: f64, y2: f64) -> Result<String, JsValue> {
    let mut doc: SketchDocument = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse: {e}")))?;
    doc.add_line_between(x1, y1, x2, y2);
    serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Serialize: {e}")))
}

#[wasm_bindgen]
pub fn sketch_add_rect(json: &str, x: f64, y: f64, w: f64, h: f64) -> Result<String, JsValue> {
    let mut doc: SketchDocument = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse: {e}")))?;
    doc.add_rectangle(x, y, w, h);
    serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Serialize: {e}")))
}

#[wasm_bindgen]
pub fn sketch_add_constraint(json: &str, constraint_json: &str) -> Result<String, JsValue> {
    let mut doc: SketchDocument = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse doc: {e}")))?;
    let c: kpe_geometry::sketch::Constraint = serde_json::from_str(constraint_json)
        .map_err(|e| JsValue::from_str(&format!("Parse constraint: {e}")))?;
    doc.add_constraint(c);
    serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Serialize: {e}")))
}

#[wasm_bindgen]
pub fn sketch_solve(json: &str) -> Result<String, JsValue> {
    let mut doc: SketchDocument = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse: {e}")))?;
    doc.solve().map_err(|e| JsValue::from_str(&e))?;
    serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Serialize: {e}")))
}

#[wasm_bindgen]
pub fn sketch_snap(json: &str, x: f64, y: f64, grid: f64) -> Result<String, JsValue> {
    let doc: SketchDocument = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse: {e}")))?;
    let result = doc.snap(x, y, grid);
    serde_json::to_string(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialize: {e}")))
}

#[wasm_bindgen]
pub fn sketch_add_circle(json: &str, cx: f64, cy: f64, radius: f64) -> Result<String, JsValue> {
    let mut doc: SketchDocument = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse: {e}")))?;
    doc.add_circle(cx, cy, radius);
    serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Serialize: {e}")))
}

#[wasm_bindgen]
pub fn sketch_add_arc(json: &str, cx: f64, cy: f64, radius: f64, start_angle: f64, end_angle: f64) -> Result<String, JsValue> {
    let mut doc: SketchDocument = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse: {e}")))?;
    doc.add_arc(cx, cy, radius, start_angle, end_angle);
    serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Serialize: {e}")))
}

#[wasm_bindgen]
pub fn sketch_remove_entity(json: &str, id: u32) -> Result<String, JsValue> {
    let mut doc: SketchDocument = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse: {e}")))?;
    doc.remove_entity(id as u64);
    serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Serialize: {e}")))
}

#[wasm_bindgen]
pub fn sketch_get_contours(json: &str) -> Result<String, JsValue> {
    let doc: SketchDocument = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse: {e}")))?;
    let contours = doc.get_contours();
    serde_json::to_string(&contours)
        .map_err(|e| JsValue::from_str(&format!("Serialize: {e}")))
}

#[wasm_bindgen]
pub fn sketch_count_dof(json: &str) -> Result<u32, JsValue> {
    let doc: SketchDocument = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse: {e}")))?;
    Ok(doc.count_dof())
}

#[wasm_bindgen]
pub fn sketch_extrude(json: &str, distance: f64) -> Result<String, JsValue> {
    let doc: SketchDocument = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse: {e}")))?;
    let (_verts, _tris) = doc.extrude_contours(distance);
    let mesh = TriangleMesh {
        vertices: _verts,
        normals: vec![],
        uvs: vec![],
        triangles: _tris,
    };
    serde_json::to_string(&mesh)
        .map_err(|e| JsValue::from_str(&format!("Serialize: {e}")))
}

// ── Original recipe functions ────────────────────────────────────

#[wasm_bindgen]
pub fn resolve_recipe(json: &str) -> Result<String, JsValue> {
    let recipe: KPERecipe = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {e}")))?;
    let solver = ParamSolver::new();
    let resolved = solver.resolve(&recipe)
        .map_err(|e| JsValue::from_str(&format!("Solver error: {e:?}")))?;
    serde_json::to_string(&resolved)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {e}")))
}

#[wasm_bindgen]
pub fn build_mesh(json: &str) -> Result<String, JsValue> {
    let recipe: KPERecipe = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {e}")))?;
    let mesh = kpe_geometry::mesh::build_mesh_from_node(&recipe.scene);
    serde_json::to_string(&mesh)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {e}")))
}

#[wasm_bindgen]
pub fn csg_union(a_json: &str, b_json: &str) -> Result<String, JsValue> {
    let a: TriangleMesh = serde_json::from_str(a_json)
        .map_err(|e| JsValue::from_str(&format!("Parse mesh A: {e}")))?;
    let b: TriangleMesh = serde_json::from_str(b_json)
        .map_err(|e| JsValue::from_str(&format!("Parse mesh B: {e}")))?;
    let kernel = CsgKernel::new();
    let result = kernel.union(&a, &b);
    serde_json::to_string(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialize: {e}")))
}

#[wasm_bindgen]
pub fn csg_subtract(a_json: &str, b_json: &str) -> Result<String, JsValue> {
    let a: TriangleMesh = serde_json::from_str(a_json)
        .map_err(|e| JsValue::from_str(&format!("Parse mesh A: {e}")))?;
    let b: TriangleMesh = serde_json::from_str(b_json)
        .map_err(|e| JsValue::from_str(&format!("Parse mesh B: {e}")))?;
    let kernel = CsgKernel::new();
    let result = kernel.subtract(&a, &b);
    serde_json::to_string(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialize: {e}")))
}

#[wasm_bindgen]
pub fn csg_intersect(a_json: &str, b_json: &str) -> Result<String, JsValue> {
    let a: TriangleMesh = serde_json::from_str(a_json)
        .map_err(|e| JsValue::from_str(&format!("Parse mesh A: {e}")))?;
    let b: TriangleMesh = serde_json::from_str(b_json)
        .map_err(|e| JsValue::from_str(&format!("Parse mesh B: {e}")))?;
    let kernel = CsgKernel::new();
    let result = kernel.intersect(&a, &b);
    serde_json::to_string(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialize: {e}")))
}

#[wasm_bindgen]
pub fn extrude_face(mesh_json: &str, face_idx: usize, distance: f64) -> Result<String, JsValue> {
    let mesh: TriangleMesh = serde_json::from_str(mesh_json)
        .map_err(|e| JsValue::from_str(&format!("Parse mesh: {e}")))?;
    let result = kpe_geometry::push_pull::extrude_face(&mesh, face_idx, distance);
    serde_json::to_string(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialize: {e}")))
}

#[wasm_bindgen]
pub fn build_mesh_from_recipe(json: &str) -> Result<String, JsValue> {
    let recipe: KPERecipe = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {e}")))?;
    let mesh = kpe_geometry::mesh::build_mesh_from_node(&recipe.scene);
    serde_json::to_string(&mesh)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {e}")))
}
