use std::collections::HashMap;

use platelet::{render_to_string, renderer::Filesystem};
use serde_json::Map;

struct MockMultiFile {
    data: HashMap<String, String>,
}

impl Filesystem for MockMultiFile {
    fn read(&self, path: &String) -> String {
        self.data.get(path).unwrap().clone()
    }
    fn move_to(&self, current: &String, path: &String) -> String {
        path.to_owned()
    }
}

#[test]
fn pl_src() {
    let vars = Map::new().into();

    let result = render_to_string(
        &vars,
        &"index.html".into(),
        &MockMultiFile {
            data: HashMap::from([
                (
                    "index.html".into(),
                    "<article><slot pl-src='embed.html'></slot></article>".to_owned(),
                ),
                ("embed.html".into(), "<p>hello world</p>".to_owned()),
            ]),
        },
    );
    assert_eq!(result.unwrap(), "<article><p>hello world</p></article>");
}

#[test]
fn pl_src_with_slot() {
    let vars = Map::new().into();

    let result = render_to_string(
            &vars,
            &"index.html".into(),
            &MockMultiFile {
                data: HashMap::from([
                    (
                        "index.html".into(),
                        "<article><slot pl-src='embed.html'><b>inner</b><b>content</b></slot></article>".to_owned(),
                    ),
                    ("embed.html".into(), "<div><slot pl-slot></slot></div>".to_owned()),
                ]),
            },
        );
    assert_eq!(
        result.unwrap(),
        "<article><div><b>inner</b><b>content</b></div></article>"
    );
}

#[test]
fn pl_src_with_named_slots() {
    let vars = Map::new().into();

    let result = render_to_string(
        &vars,
        &"index.html".into(),
        &MockMultiFile {
            data: HashMap::from([
                (
                    "index.html".into(),
                    "<slot pl-src='embed.html'>\
                             <template pl-slot='left'><b>Left</b> hand side</template>\
                             <template pl-slot='right'><b>Right</b> hand side</template>\
                         </slot>"
                        .to_owned(),
                ),
                (
                    "embed.html".into(),
                    "<left><slot pl-slot='left'></slot></left>\
                         <right><slot pl-slot='right'></slot></right>"
                        .to_owned(),
                ),
            ]),
        },
    );
    assert_eq!(
        result.unwrap(),
        "<left><b>Left</b> hand side</left><right><b>Right</b> hand side</right>"
    );
}

#[test]
fn pl_src_with_cotext() {
    let vars = Map::new().into();

    let result = render_to_string(
        &vars,
        &"index.html".into(),
        &MockMultiFile {
            data: HashMap::from([
                (
                    "index.html".into(),
                    r#"<slot pl-src='embed.html' ^message='"hello world"'></slot>"#.to_owned(),
                ),
                ("embed.html".into(), "<code>{{message}}</code>".to_owned()),
            ]),
        },
    );
    assert_eq!(result.unwrap(), "<code>hello world</code>");
}
