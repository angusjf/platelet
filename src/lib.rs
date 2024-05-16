use serde_json::Value;
use wasm_bindgen::prelude::wasm_bindgen;

mod expression_eval;
mod expression_parser;
mod for_loop_parser;
mod for_loop_runner;
mod html;
mod html_parser;
mod rcdom;
pub mod renderer;
mod text_node;

#[wasm_bindgen]
pub fn render(html: String, context: String) -> String {
    let context: Value = serde_json::from_str(&context).unwrap();

    renderer::render_string(&context, html)
}

pub fn render_string(html: String, context: &Value) -> String {
    renderer::render_string(&context, html)
}
