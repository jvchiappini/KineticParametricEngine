use wasm_bindgen::prelude::*;
use kpe_schema::recipe::KPERecipe;
use kpe_parametric::Solver;
use kpe_geometry::mesh::MeshBuilder;

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

    let builder = MeshBuilder::new();
    let mesh = builder.build_from_node(&recipe.scene);

    serde_json::to_string(&mesh)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {e}")))
}

#[wasm_bindgen]
pub fn hello() -> String {
    "KPE Engine ready".to_string()
}
