use kuchikiki::{traits::*, NodeData, NodeRef};
use markup5ever::{namespace_url, ns, LocalName, QualName};
use serde_json::Value;
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

use crate::expression_eval::{eval, truthy};
use crate::expression_parser::expr;
use crate::text_node::render_text_node;

const USAGE: &str = "echo '{ \"some\": \"args\" }' | platelet [template.html]";

pub enum SrcAndData {
    BothSrcAndData(String, Value),
    JustSrc(String),
    JustData(Value),
}

enum Replacement {
    Template(SrcAndData),
}

pub trait Filesystem {
    fn get_data_at_path(&self, path: &PathBuf) -> String;
}

pub fn modifyNode<F>(node: &mut NodeRef, vars: &Value, filename: &PathBuf, filesystem: &F)
where
    F: Filesystem,
{
    match node.data() {
        NodeData::Element(e) => {
            let mut attrs = e.attributes.borrow_mut();

            if let Some(exp) = attrs.get("pl-if") {
                let mut exp = exp;
                match expr(&mut exp) {
                    Ok(exp) => match eval(&exp, vars) {
                        Ok(v) => {
                            if !truthy(&v) {
                                node.detach()
                            }
                            attrs.remove("pl-if");
                        }
                        Err(e) => todo!(),
                    },
                    Err(x) => todo!(),
                }
            }

            if let Some(exp) = attrs.get("pl-html") {
                let mut exp = exp;
                match expr(&mut exp) {
                    Ok(exp) => match eval(&exp, vars) {
                        Ok(Value::String(html)) => {
                            if e.name.local == "template".to_string() {
                                let new_child = xyz(&html);
                                node.insert_after(new_child);
                                node.detach()
                            } else {
                                attrs.remove("pl-html");

                                for child in node.children() {
                                    child.detach()
                                }
                                let new_child = xyz(&html);
                                node.append(new_child);
                            }
                        }
                        Ok(_) => todo!(),
                        Err(e) => todo!(),
                    },
                    Err(x) => todo!(),
                }
            }
        }
        NodeData::Text(t) => {
            let mut t = t.borrow_mut();
            match render_text_node(t.as_ref(), &vars) {
                Ok(content) => {
                    let content = content.to_string();
                    *t = content;
                }
                Err(e) => panic!("{:?}", e),
            }
        }
        NodeData::Comment(_) => {}
        NodeData::ProcessingInstruction(_) => {}
        NodeData::Doctype(_) => {}
        NodeData::Document(_) => {}
        NodeData::DocumentFragment => {}
    }

    for mut child in node.children() {
        modifyNode(&mut child, vars, filename, filesystem);
    }
}

fn xyz(html: &str) -> NodeRef {
    kuchikiki::parse_fragment(
        QualName::new(None, ns!(html), LocalName::from("body")),
        vec![],
    )
    .one(html)
}

pub fn render<F>(vars: &Value, filename: &PathBuf, filesystem: &F) -> String
where
    F: Filesystem,
{
    let html = filesystem.get_data_at_path(filename);
    let mut node = xyz(&html);

    modifyNode(&mut node, vars, filename, filesystem);

    let mut writer = vec![];

    node.serialize(&mut writer).unwrap();

    String::from_utf8(writer)
        .unwrap()
        .replace("<html>", "")
        .replace("</html>", "")
        .into()
}

#[cfg(test)]
mod test {

    use serde_json::{json, Map};

    use super::*;

    struct MockFilesystem {
        data: String,
    }

    impl Filesystem for MockFilesystem {
        fn get_data_at_path(&self, _: &PathBuf) -> String {
            self.data.clone()
        }
    }

    #[test]
    fn templateless_text_node() {
        let vars = json!({ "hello": "world" });

        let result = render(
            &vars,
            &PathBuf::new(),
            &MockFilesystem {
                data: "<h1>nothing here</h1>".into(),
            },
        );
        assert_eq!(result, "<h1>nothing here</h1>");
    }

    #[test]
    #[ignore]
    fn templateless_html_doc() {
        let vars = json!({ "hello": "world" });

        let result = render(
            &vars,
            &PathBuf::new(),
            &MockFilesystem {
                data: "<!doctype html><html><head><title>a</title></head><body></body></html>"
                    .into(),
            },
        );
        assert_eq!(
            result,
            "<!doctype html><html><head><title>a</title></head><body></body></html>"
        );
    }

    #[test]
    fn templated_text_node() {
        let vars = json!({ "hello": "world" });

        let result = render(
            &vars,
            &PathBuf::new(),
            &MockFilesystem {
                data: "<h1>{{hello}}</h1>".into(),
            },
        );
        assert_eq!(result, "<h1>world</h1>");
    }

    #[test]
    fn complex_text_node() {
        let vars = json!({ "user": {"firstname": "Yuri", "lastname" : "Gagarin" } });

        let result = render(
            &vars,
            &PathBuf::new(),
            &MockFilesystem {
                data: "<h1>Dear {{user.firstname}} {{user.lastname}},</h1>".into(),
            },
        );
        assert_eq!(result, "<h1>Dear Yuri Gagarin,</h1>");
    }

    #[test]
    fn text_node_with_expressions() {
        let vars = json!({ "countries": [ "portugal" ] });

        let result = render(
            &vars,
            &PathBuf::new(),
            &MockFilesystem {
                data: "<h1>{{countries[0]}} {{ 1 + 2 }}</h1>".into(),
            },
        );
        assert_eq!(result, "<h1>portugal 3</h1>");
    }

    #[test]
    fn pl_if() {
        let vars = Map::new().into();

        let result = render(
            &vars,
            &PathBuf::new(),
            &MockFilesystem {
                data: "<p>this</p>\
                    <p pl-if='false'>not this</p>\
                    <p>not this</p>"
                    .into(),
            },
        );
        assert_eq!(result, "<p>this</p><p>not this</p>");
    }

    // #[test]
    // fn pl_is() {
    //     let vars = json!({ "header": true });

    //     let result = render(
    //         &vars,
    //         &PathBuf::new(),
    //         &MockFilesystem {
    //             data: "<p pl-is='header ? \"h1\" : \"h2\"'>this</p>".into(),
    //         },
    //     );
    //     assert_eq!(result, "<h1>this</h1>");
    // }

    #[test]
    fn pl_html() {
        let vars = json!({ "content": "<p>hello world</p>" });

        let result = render(
            &vars,
            &PathBuf::new(),
            &MockFilesystem {
                data: "<article pl-html='content'>something that used to be here</article>".into(),
            },
        );
        assert_eq!(result, "<article><p>hello world</p></article>");
    }

    #[test]
    fn pl_html_with_template() {
        let vars = json!({ "content": "<p>hello world</p>" });

        let result = render(
            &vars,
            &PathBuf::new(),
            &MockFilesystem {
                data: "<template pl-html='content'>something that used to be here</template>"
                    .into(),
            },
        );
        assert_eq!(result, "<p>hello world</p>");
    }
}
