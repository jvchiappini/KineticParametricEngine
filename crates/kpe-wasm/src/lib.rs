pub mod api;
pub mod convert;

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
