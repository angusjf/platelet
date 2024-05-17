use std::collections::HashMap;

use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn render_files(filename: String, files: String, context: String) -> Result<String, String> {
    // let context: Value = serde_json::from_str(&context).unwrap();

    // platelet::render_to_string(&context, html).map_err(|e| e.to_string())
    Ok(String::new())
}
