use std::collections::HashMap;

use serde_json::Value;
use wasm_bindgen::prelude::wasm_bindgen;

use platelet::renderer::Filesystem;

struct FileMap {
    files: HashMap<String, String>,
}

impl Filesystem for FileMap {
    fn move_to(&self, _current: &String, path: &String) -> String {
        path.to_owned()
    }

    fn read(&self, file: &String) -> String {
        self.files.get(file).unwrap().to_owned()
    }
}

#[wasm_bindgen]
pub fn render_files(filename: String, files: String, context: String) -> Result<String, String> {
    let context: Value = serde_json::from_str(&context).unwrap();

    let filesystem: HashMap<String, String> = serde_json::from_str(&files).unwrap();

    let filesystem = FileMap { files: filesystem };

    platelet::render_to_string(&context, &filename, &filesystem).map_err(|e| e.to_string())
}
