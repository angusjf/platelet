use renderer::RenderError;
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
pub fn render(html: String, context: String) -> Result<String, String> {
    let context: Value = serde_json::from_str(&context).unwrap();

    renderer::render_string(&context, html).map_err(|e| e.to_string())
}

pub fn render_string(html: String, context: &Value) -> Result<String, RenderError> {
    renderer::render_string(&context, html)
}

#[cfg(test)]
mod render_test {

    use super::*;

    #[test]
    fn happy_path() {
        let result = render(
            "<h1>{{ hello }} world".to_owned(),
            r#"{ "hello": "hi" }"#.to_owned(),
        );
        assert_eq!(result, Ok("<h1>hi world</h1>".to_owned()));
    }

    #[test]
    fn parser_error() {
        let result = render(
            "<h1 pl-for='x, in [1,2,3]'>{{ hello }} world {{ x }}".to_owned(),
            r#"{ "hello": "hi" }"#.to_owned(),
        );
        assert_eq!(
            result,
            Err(r#"FOR LOOP PARSER ERROR:
x, in [1,2,3]
^
invalid for loop"#
                .to_owned())
        );
    }
}
