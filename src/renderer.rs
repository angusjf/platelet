use markup5ever::{namespace_url, ns, LocalName, QualName};
use serde::{Deserialize, Deserializer, Serialize};

use core::fmt;
use serde_json::Value;
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use crate::expression_eval::{eval, truthy};
use crate::expression_parser::expr;
use crate::for_loop_parser::for_loop;
use crate::text_node::render_text_node;
use crate::{deno_dom, for_loop_runner};

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

#[derive(Debug, Clone)]
enum Node {
    Small {
        id: u64,
        content: String,
    },
    Big {
        id: u64,
        name: String,
        attrs: HashMap<String, String>,
        children: Vec<Node>,
    },
}

// type node = [NodeType, nodeName, attributes, node[]]
//             | [NodeType, characterData]
fn node_from_array(val: &Value) -> Node {
    let val = val.as_array().unwrap();

    if val.len() == 2 {
        Node::Small {
            id: val[0].as_u64().unwrap(),
            content: val[1].as_str().unwrap().to_owned(),
        }
    } else {
        Node::Big {
            id: val[0].as_u64().unwrap(),
            name: val[1].as_str().unwrap().to_owned(),
            attrs: val[2]
                .as_array()
                .unwrap()
                .iter()
                .map(|attr| {
                    let attr = attr.as_array().unwrap();
                    (
                        attr[0].as_str().unwrap().to_owned(),
                        attr[1].as_str().unwrap().to_owned(),
                    )
                })
                .collect(),
            children: val[3..].iter().map(|x| node_from_array(&x)).collect(),
        }
    }
}

enum Cmd {
    Nothing,
    DeleteMe,
    Loop(Vec<Value>),
}

impl Node {
    fn to_string(&self) -> String {
        match self {
            Node::Small { content, id: _id } => content.clone(),
            Node::Big {
                id: _,
                name,
                attrs,
                children,
            } => {
                let attrs_str = attrs
                    .iter()
                    .map(|(key, value)| format!(" {}=\"{}\"", key, value))
                    .collect::<String>();

                let children_str = children
                    .iter()
                    .map(|child| child.to_string())
                    .collect::<String>();

                format!("<{}{}>{}</{}>", name, attrs_str, children_str, name)
            }
        }
    }
}

fn render_elem<F>(node: &mut Node, vars: &Value, filename: &PathBuf, filesystem: &F) -> Cmd
where
    F: Filesystem,
{
    match node {
        Node::Small { content: t, .. } => match render_text_node(t.as_ref(), &vars) {
            Ok(content) => {
                let content = content.to_string();
                *t = content;
                Cmd::Nothing
            }
            Err(e) => panic!("{:?}", e),
        },
        Node::Big {
            attrs, children, ..
        } => {
            let mut cmd = Cmd::Nothing;
            if let Some(exp) = attrs.get("pl-if") {
                match expr(&mut exp.as_ref()) {
                    Ok(exp) => match eval(&exp, vars) {
                        Ok(v) => {
                            attrs.remove("pl-if");
                            if !truthy(&v) {
                                return Cmd::DeleteMe;
                            }
                        }
                        Err(e) => todo!(),
                    },
                    Err(x) => todo!(),
                }
            }

            if let Some(fl) = attrs.get("pl-for") {
                let fl = fl.clone();
                attrs.remove("pl-for");
                match for_loop(&mut fl.as_ref()) {
                    Ok(fl) => match for_loop_runner::for_loop_runner(&fl, vars) {
                        Ok(contexts) => return Cmd::Loop(contexts),
                        Err(_e) => todo!(),
                    },
                    Err(_x) => todo!(),
                }
            }

            let mut i = 0;

            while i < children.len() {
                let child = children[i].borrow_mut();
                match render_elem(child, vars, filename, filesystem) {
                    Cmd::DeleteMe => {
                        children.remove(i);
                    }
                    Cmd::Loop(contexts) => {
                        let child = children.remove(i);
                        for ctx in contexts {
                            let mut child = child.clone();
                            render_elem(&mut child, &ctx, filename, filesystem);
                            children.insert(i, child);
                            i += 1;
                        }
                    }
                    Cmd::Nothing => {
                        i += 1;
                    }
                };
            }

            return cmd;
        }
    }
}

fn parse_html(html: String) -> Node {
    let node = deno_dom::parse_frag(html, "body".to_owned());
    let node = serde_json::from_str(&node).unwrap();
    let node = node_from_array(&node);
    node
}

pub fn render<F>(vars: &Value, filename: &PathBuf, filesystem: &F) -> String
where
    F: Filesystem,
{
    let html = filesystem.get_data_at_path(filename);

    let mut node = parse_html(html);

    render_elem(&mut node, vars, filename, filesystem);

    node.to_string()
        .replace("<#document>", "")
        .replace("<html>", "")
        .replace("</html>", "")
        .replace("</#document>", "")
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

    #[test]
    #[ignore]
    fn pl_is() {
        let vars = json!({ "header": true });

        let result = render(
            &vars,
            &PathBuf::new(),
            &MockFilesystem {
                data: "<p pl-is='header ? \"h1\" : \"h2\"'>this</p>".into(),
            },
        );
        assert_eq!(result, "<h1>this</h1>");
    }

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

    #[test]
    #[ignore]
    fn template_preserved() {
        let vars = Map::new().into();

        let result = render(
            &vars,
            &PathBuf::new(),
            &MockFilesystem {
                data: "<template><h1>hello</h1></template>".into(),
            },
        );
        assert_eq!(result, "<template><h1>hello</h1></template>");
    }

    #[test]
    fn pl_for() {
        let vars = Map::new().into();

        let result = render(
            &vars,
            &PathBuf::new(),
            &MockFilesystem {
                data: "<div><p pl-for='x in [1,2,3]'>{{x}}</p></div>".into(),
            },
        );
        assert_eq!(result, "<div><p>1</p><p>2</p><p>3</p></div>");
    }

    #[test]
    #[ignore]
    fn pl_for_template() {
        let vars = Map::new().into();

        let result = render(
            &vars,
            &PathBuf::new(),
            &MockFilesystem {
                data: "<div><template pl-for='x in [1,2,3]'><p>{{x}}</p></template></div>".into(),
            },
        );
        assert_eq!(result, "<div><p>1</p><p>2</p><p>3</p></div>");
    }
}
