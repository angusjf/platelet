use std::collections::HashMap;

use serde_json::Value;
use wasm_bindgen::prelude::wasm_bindgen;

use platelet::renderer::Filesystem;

struct FileMap {
    files: HashMap<String, String>,
}

impl Filesystem<String> for FileMap {
    fn move_to(&self, _current: &String, path: &String) -> Result<String, String> {
        if self.files.contains_key(path) {
            Ok(path.to_owned())
        } else {
            Err(path.to_owned() + " does not exist")
        }
    }

    fn read(&self, file: &String) -> Result<String, String> {
        Ok(self
            .files
            .get(file)
            .ok_or(format!("FILESYSTEM ERROR: Could not find file `{}`", file))?
            .to_owned())
    }
}

#[wasm_bindgen]
pub fn render_files(filename: String, files: String, context: String) -> Result<String, String> {
    let context: Value = serde_json::from_str(&context)
        .map_err(|e| "Could not deserialize: ".to_owned() + &e.to_string())?;

    let filesystem: HashMap<String, String> = serde_json::from_str(&files).unwrap();

    let filesystem = FileMap { files: filesystem };

    platelet::render_with_custom_filesystem(&filename, &context, &filesystem)
        .map_err(|e| e.to_string())
}
