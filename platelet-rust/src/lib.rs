use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    rc::Rc,
};

use renderer::{render, Filesystem, RenderError};
use serde_json::Value;

mod expression_eval;
mod expression_parser;
mod for_loop_parser;
mod for_loop_runner;
mod html;
mod html_parser;
mod rcdom;
pub mod renderer;
mod text_node;
mod types;

pub fn render_to_string<F>(
    vars: &Value,
    filename: &String,
    filesystem: &F,
) -> Result<String, RenderError>
where
    F: Filesystem,
{
    render(
        vars,
        Rc::new(HashMap::new()),
        &mut HashSet::new(),
        &filename,
        filesystem,
    )
    .map(|x| x.to_string())
}

pub(crate) struct SingleFile {
    data: String,
}

impl Filesystem for SingleFile {
    fn move_to(&self, _current: &String, to: &String) -> String {
        to.to_owned()
    }
    fn read(&self, _path: &String) -> String {
        self.data.clone()
    }
}

pub fn render_string_to_string(vars: &Value, html: String) -> Result<String, RenderError> {
    render_to_string(&vars, &"".to_owned(), &SingleFile { data: html })
}

struct PathFilesystem {}

impl Filesystem for PathFilesystem {
    fn read(&self, filename: &String) -> String {
        let filename: PathBuf = filename.try_into().unwrap();
        let mut file = File::open(filename.clone()).expect("bad file");
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        buf
    }
    fn move_to(&self, current: &String, path: &String) -> String {
        let current: PathBuf = current.try_into().unwrap();
        current
            .parent()
            .unwrap()
            .join(path)
            .to_str()
            .unwrap()
            .to_owned()
    }
}

pub fn render_with_filesystem(vars: &Value, filename: &Path) -> Result<String, RenderError> {
    render_to_string(
        &vars,
        &filename.to_str().unwrap().to_owned(),
        &PathFilesystem {},
    )
}

#[cfg(test)]
mod render_test {

    use serde_json::json;

    use super::*;

    #[test]
    fn happy_path() {
        let result = render_string_to_string(
            &json!({ "hello": "hi" }),
            "<h1>{{ hello }} world".to_owned(),
        );
        assert_eq!(result, Ok("<h1>hi world</h1>".to_owned()));
    }

    #[test]
    fn for_loop_parser_error() {
        let result = render_string_to_string(
            &json!({ "hello": "hi" }),
            "<h1 pl-for='x, in [1,2,3]'>{{ hello }} world {{ x }}".to_owned(),
        );
        assert_eq!(
            result.unwrap_err().to_string(),
            (r#"FOR LOOP PARSER ERROR:
x, in [1,2,3]
^
invalid for loop"#
                .to_owned())
        );
    }

    #[test]
    fn for_loop_exec_error() {
        let result = render_string_to_string(
            &json!({ "hello": "hi" }),
            "<h1 pl-for='x in 1'>{{ hello }} world {{ x }}".to_owned(),
        );
        assert_eq!(
            result.unwrap_err().to_string(),
            (r#"FOR LOOP EVALUATION ERROR: Expected array, found number"#.to_owned())
        );
    }
}
