use wasm_bindgen::prelude::*;
use kpe_schema::recipe::KPERecipe;
use kpe_schema::geometry::TriangleMesh;
use kpe_parametric::Solver;
use kpe_geometry::csg::CsgKernel;

#[wasm_bindgen]
pub fn resolve_recipe(json: &str) -> Result<String, JsValue> {
    let recipe: KPERecipe = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {e}")))?;

    let solver = Solver::new();
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
pub fn hello() -> String {
    "KPE Engine ready".to_string()
}
