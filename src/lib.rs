use serde_json::Value;
use wasm_bindgen::prelude::wasm_bindgen;

mod deno_dom;
mod expression_eval;
mod expression_parser;
mod for_loop_parser;
mod for_loop_runner;
mod rcdom;
pub mod renderer;
mod text_node;

#[wasm_bindgen]
pub fn render(html: String, context: String) -> String {
    let context: Value = serde_json::from_str(&context).unwrap();

    renderer::render_string(&context, html)
}
