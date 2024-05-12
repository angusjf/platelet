use serde_json::Value;
use std::borrow::BorrowMut;
use std::path::PathBuf;

use crate::expression_eval::{eval, truthy};
use crate::expression_parser::expr;
use crate::for_loop_parser::for_loop;
use crate::text_node::render_text_node;
use crate::{deno_dom, for_loop_runner};

pub trait Filesystem {
    fn get_data_at_path(&self, path: &PathBuf) -> String;
}

#[derive(Debug, Clone, PartialEq)]
enum Node {
    Text {
        content: String,
    },
    Element {
        name: String,
        attrs: Vec<(String, String)>,
        children: Vec<Node>,
    },
    Comment {
        content: String,
    },
    Document {
        children: Vec<Node>,
    },
    Doctype {
        doctype: String,
    },
}

// type node = [NodeType, nodeName, attributes, node[]]
//             | [NodeType, characterData]
fn node_from_array(val: &Value) -> Node {
    let val = val.as_array().unwrap();

    let id = val[0].as_u64().unwrap();

    match id {
        0 => panic!("node id 0 is undefined"),
        2 => panic!("node id 2 is for attributes"),
        4 => panic!("unexpected CDATA section"),
        7 => panic!("unexpected PROCESSING_INSTRUCTION_NODE section"),
        9 => Node::Document {
            children: val[3..].iter().map(|x| node_from_array(&x)).collect(),
        },
        1 => Node::Element {
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
        },
        10 => Node::Doctype {
            doctype: val[1].as_str().unwrap().to_owned(),
        },

        3 => Node::Text {
            content: val[1].as_str().unwrap().to_owned(),
        },
        8 => Node::Comment {
            content: val[1].as_str().unwrap().to_owned(),
        },
        11 => panic!("unexpected DOCUMENT_FRAGMENT_NODE"),
        5 | 6 | 12.. => panic!("node ids 5, 6 and above 12 are not in the spec"),
    }
}

enum Cmd {
    Nothing,
    DeleteMe,
    Loop(Vec<Value>),
    ReplaceMeWith(Node),
    ChildrenOnly,
}

impl Node {
    fn to_string(&self) -> String {
        fn html_text_safe(s: &str) -> String {
            s.replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
            // .replace('"', "&quot;")
            // .replace("'", "&#39;")
            // .replace('/', "&#x2F;")
            // .replace('`', "&#x60;")
            // .replace('=', "&#x3D;")
            //  TODO sort this mess
        }
        fn html_attr_safe(s: &str) -> String {
            s.replace('"', "&quot;").replace("'", "&#39;")
            // .replace('/', "&#x2F;")
            // .replace('`', "&#x60;")
            // .replace('=', "&#x3D;")
            //  TODO sort this mess
        }
        match self {
            Node::Doctype { doctype } => format!("<!DOCTYPE {}>", html_text_safe(doctype)),
            Node::Comment { content } => format!("<!--{}-->", html_text_safe(content)),
            Node::Text { content } => html_text_safe(content),
            Node::Element {
                name,
                attrs,
                children,
            } => {
                let attrs_str = attrs
                    .iter()
                    .map(|(key, value)| format!(" {}='{}'", key, html_attr_safe(value)))
                    .collect::<String>();

                let children_str = children
                    .iter()
                    .map(|child| child.to_string())
                    .collect::<String>();

                match name.as_str() {
                    "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link"
                    | "meta" | "param" | "source" | "track" | "wbr" => {
                        format!("<{}{}>", name, attrs_str)
                    }
                    _ => {
                        format!("<{}{}>{}</{}>", name, attrs_str, children_str, name)
                    }
                }
            }
            Node::Document { children } => children.iter().map(|child| child.to_string()).collect(),
        }
    }
}

fn render_elem<F>(
    node: &mut Node,
    vars: &Value,
    previous_conditional: &Option<bool>,
    next_neighbour_conditional: &mut Option<bool>,
    filename: &PathBuf,
    filesystem: &F,
) -> Cmd
where
    F: Filesystem,
{
    match node {
        Node::Doctype { .. } => return Cmd::Nothing,
        Node::Document { children } => {
            render_children(children, vars, filename, filesystem);
            return Cmd::Nothing;
        }
        Node::Comment { .. } => return Cmd::Nothing,
        Node::Text { content: t, .. } => match render_text_node(t.as_ref(), &vars) {
            Ok(content) => {
                let content = content.to_string();
                *t = content;
                Cmd::Nothing
            }
            Err(e) => panic!("{:?}", e),
        },
        Node::Element {
            attrs: attrs_list,
            children,
            name,
            ..
        } => {
            // if !(attrsList.re.contains_key("pl-if") || attrs.contains_key("pl-else-if")) {
            //     *next_neighbour_conditional = None;
            // }

            if let Some(exp_index) = attrs_list.iter().position(|(name, _)| name == "pl-if") {
                let (_, exp) = &attrs_list[exp_index];
                match expr(&mut exp.as_ref()) {
                    Ok(exp) => match eval(&exp, vars) {
                        Ok(v) => {
                            let cond = truthy(&v);
                            *next_neighbour_conditional = Some(cond);
                            if !cond {
                                return Cmd::DeleteMe;
                            } else if name == "template" {
                                return Cmd::ChildrenOnly;
                            }
                            attrs_list.remove(exp_index);
                        }
                        Err(_e) => todo!(),
                    },
                    Err(x) => panic!("{:?}", x),
                }
            }

            if let Some(exp_index) = attrs_list.iter().position(|(name, _)| name == "pl-else-if") {
                let (_, exp) = &attrs_list[exp_index];
                match previous_conditional {
                    Some(true) => {
                        *next_neighbour_conditional = Some(true);
                        return Cmd::DeleteMe;
                    }
                    Some(false) => match expr(&mut exp.as_ref()) {
                        Ok(exp) => match eval(&exp, vars) {
                            Ok(v) => {
                                let cond = truthy(&v);
                                *next_neighbour_conditional = Some(cond);
                                if !cond {
                                    return Cmd::DeleteMe;
                                }
                                attrs_list.remove(exp_index);
                            }
                            Err(_e) => todo!(),
                        },
                        Err(_x) => todo!(),
                    },
                    None => todo!(),
                }
            }

            if let Some(index) = attrs_list.iter().position(|(name, _)| name == "pl-else") {
                match previous_conditional {
                    Some(true) => {
                        return Cmd::DeleteMe;
                    }
                    Some(false) => {
                        attrs_list.remove(index);
                    }
                    None => todo!(),
                }
            }

            if let Some(fl_index) = attrs_list.iter().position(|(name, _)| name == "pl-for") {
                let (_, fl) = &attrs_list[fl_index];

                match for_loop(&mut fl.as_ref()) {
                    Ok(fl) => match for_loop_runner::for_loop_runner(&fl, vars) {
                        Ok(contexts) => {
                            attrs_list.remove(fl_index);
                            return Cmd::Loop(contexts);
                        }
                        Err(_e) => todo!(),
                    },
                    Err(_x) => todo!(),
                }
            }

            if let Some(exp_index) = attrs_list.iter().position(|(name, _)| name == "pl-html") {
                let (_, exp) = &attrs_list[exp_index];

                match expr(&mut exp.as_ref()) {
                    Ok(exp) => match eval(&exp, vars) {
                        Ok(Value::String(html)) => {
                            let node = parse_html(html);
                            if name == "template" {
                                return Cmd::ReplaceMeWith(node);
                            } else {
                                attrs_list.remove(exp_index);
                                children.clear();
                                children.push(node);
                                return Cmd::Nothing;
                            }
                        }
                        Ok(_v) => {
                            todo!()
                        }
                        Err(_e) => todo!(),
                    },
                    Err(_x) => todo!(),
                }
            }

            if let Some(src_index) = attrs_list.iter().position(|(name, _)| name == "pl-src") {
                let (_, src) = &attrs_list[src_index];

                let path = filename.parent().unwrap().join(src);
                let rendered = render(vars, &path, filesystem);
                return Cmd::ReplaceMeWith(rendered);
            }

            if let Some(exp_index) = attrs_list.iter().position(|(name, _)| name == "pl-is") {
                let (_, exp) = &attrs_list[exp_index];

                match expr(&mut exp.as_ref()) {
                    Ok(exp) => match eval(&exp, vars) {
                        Ok(Value::String(tag)) => {
                            attrs_list.remove(exp_index);
                            *name = tag;
                        }
                        Ok(_v) => {
                            todo!()
                        }
                        Err(_e) => todo!(),
                    },
                    Err(_x) => todo!(),
                }
            }

            modify_attrs(attrs_list, vars);

            render_children(children, vars, filename, filesystem);

            return Cmd::Nothing;
        }
    }
}

fn render_children<F>(children: &mut Vec<Node>, vars: &Value, filename: &PathBuf, filesystem: &F)
where
    F: Filesystem,
{
    let mut i = 0;

    let mut get_this = None;
    let mut set_this = None;
    while i < children.len() {
        let child = children[i].borrow_mut();
        match render_elem(child, vars, &get_this, &mut set_this, filename, filesystem) {
            Cmd::DeleteMe => {
                children.remove(i);
            }
            Cmd::Loop(contexts) => {
                let child = children.remove(i);
                for ctx in contexts {
                    let mut child = child.clone();
                    render_elem(
                        &mut child,
                        &ctx,
                        &get_this,
                        &mut set_this,
                        filename,
                        filesystem,
                    );
                    children.insert(i, child);
                    i += 1;
                }
            }
            Cmd::Nothing => {
                i += 1;
            }
            Cmd::ReplaceMeWith(node) => {
                children[i] = node;
                i += 1;
            }
            Cmd::ChildrenOnly => {
                let child = children.remove(i);
                let children_ = match child {
                    Node::Element { children, .. } => children,
                    _ => panic!(),
                };
                for mut child_ in children_ {
                    render_elem(
                        &mut child_,
                        &vars,
                        &get_this,
                        &mut set_this,
                        filename,
                        filesystem,
                    );
                    children.insert(i, child_);
                    i += 1;
                }
            }
        };
        get_this = set_this;
    }
}

fn attrify(val: &Value) -> Option<String> {
    match val {
        Value::Null => None,
        Value::Bool(false) => None,
        Value::Bool(true) => Some("true".into()),
        Value::Number(n) => Some(n.to_string()),
        Value::String(s) => Some(s.to_string()),
        Value::Array(a) => {
            let xs: Vec<_> = a.iter().filter_map(attrify).collect();

            Some(xs.join(" "))
        }
        Value::Object(o) => {
            let xs: Vec<_> = o
                .iter()
                .filter_map(|(k, v)| if truthy(v) { Some(k.to_owned()) } else { None })
                .collect();
            Some(xs.join(" "))
        }
    }
}

fn modify_attrs(attrs: &mut Vec<(String, String)>, vars: &Value) {
    attrs.retain_mut(|(name_original, val)| {
        if let Some(name) = name_original.strip_prefix('^') {
            let exp = expr(&mut val.as_ref()).unwrap();
            let v = eval(&exp, vars).unwrap();
            match attrify(&v) {
                None => false,
                Some(s) => {
                    *name_original = name.to_owned();
                    *val = s;
                    true
                }
            }
        } else {
            true
        }
    });

    // let mut delete_me = vec![];
    // let mut to_insert: HashMap<String, String> = HashMap::new();

    // for (c_name, val) in attrs.clone() {
    //
    //         let name = name.to_owned();
    //         delete_me.push(c_name.clone());
    //
    //
    //
    //
    //     }
    // }

    // for name in delete_me.iter() {
    //     attrs.remove(name).unwrap();
    // }

    // for (attr, v) in to_insert.drain() {
    //     attrs.insert(attr, v);
    // }
}

fn parse_html(html: String) -> Node {
    let full_doc = {
        let html = html.to_lowercase();
        html.contains("<html")
            || html.contains("<body")
            || html.contains("<head")
            || html.contains("<!DOCTYPE")
    };

    let node = if full_doc {
        deno_dom::parse(html)
    } else {
        deno_dom::parse_frag(html, "body".to_owned())
    };
    let node = serde_json::from_str(&node).unwrap();
    let node = node_from_array(&node);

    if full_doc {
        node
    } else {
        match node {
            Node::Document { ref children } => match &children[..] {
                [Node::Element {
                    ref name,
                    ref attrs,
                    children,
                }] if name == "html" && attrs.is_empty() => Node::Document {
                    children: children.clone(),
                },
                _ => node,
            },
            _ => node,
        }
    }
}

fn render<F>(vars: &Value, filename: &PathBuf, filesystem: &F) -> Node
where
    F: Filesystem,
{
    let html = filesystem.get_data_at_path(filename);

    let mut node = parse_html(html);

    render_elem(&mut node, vars, &None, &mut None, filename, filesystem);
    node
}

pub fn render_to_string<F>(vars: &Value, filename: &PathBuf, filesystem: &F) -> String
where
    F: Filesystem,
{
    render(vars, filename, filesystem).to_string()
}

pub fn render_string(vars: &Value, html: String) -> String {
    render_to_string(&vars, &PathBuf::new(), &MockSingleFile { data: html })
}

struct MockSingleFile {
    data: String,
}

impl Filesystem for MockSingleFile {
    fn get_data_at_path(&self, _: &PathBuf) -> String {
        self.data.clone()
    }
}

#[cfg(test)]
mod render_test {

    use std::collections::HashMap;

    use serde_json::{json, Map};

    use super::*;

    struct MockMultiFile {
        data: HashMap<PathBuf, String>,
    }

    impl Filesystem for MockMultiFile {
        fn get_data_at_path(&self, path: &PathBuf) -> String {
            self.data.get(path).unwrap().clone()
        }
    }

    #[test]
    fn templateless_text_node() {
        let vars = json!({ "hello": "world" });

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: "<h1>nothing here</h1>".into(),
            },
        );
        assert_eq!(result, "<h1>nothing here</h1>");
    }

    #[test]
    fn templateless_html_doc() {
        let vars = json!({ "hello": "world" });

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: "<!doctype html><html><head><title>a</title></head><body></body></html>"
                    .into(),
            },
        );
        assert_eq!(
            result,
            "<!DOCTYPE html><html><head><title>a</title></head><body></body></html>"
        );
    }

    #[test]
    fn templated_text_node() {
        let vars = json!({ "hello": "world" });

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: "<h1>{{hello}}</h1>".into(),
            },
        );
        assert_eq!(result, "<h1>world</h1>");
    }

    #[test]
    fn complex_text_node() {
        let vars = json!({ "user": {"firstname": "Yuri", "lastname" : "Gagarin" } });

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: "<h1>Dear {{user.firstname}} {{user.lastname}},</h1>".into(),
            },
        );
        assert_eq!(result, "<h1>Dear Yuri Gagarin,</h1>");
    }

    #[test]
    fn text_node_with_expressions() {
        let vars = json!({ "countries": [ "portugal" ] });

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: "<h1>{{countries[0]}} {{ 1 + 2 }}</h1>".into(),
            },
        );
        assert_eq!(result, "<h1>portugal 3</h1>");
    }

    #[test]
    fn pl_if() {
        let vars = Map::new().into();

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: "<p>this</p>\
                    <p pl-if='false'>not this</p>\
                    <p>also this</p>"
                    .into(),
            },
        );
        assert_eq!(result, "<p>this</p><p>also this</p>");
    }

    #[test]
    #[ignore]
    fn pl_else_if_true() {
        let vars = Map::new().into();

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: "<p>this</p>\
                        <p pl-if='false'>not this</p>\
                        <p pl-else-if='true'>also this</p>"
                    .into(),
            },
        );
        assert_eq!(result, "<p>this</p><p>also this</p>");
    }

    #[test]
    fn pl_else_if_false() {
        let vars = Map::new().into();

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: "<p>this</p>\
                        <p pl-if='false'>not this</p>\
                        <p pl-else-if='false'>also this</p>"
                    .into(),
            },
        );
        assert_eq!(result, "<p>this</p>");
    }

    #[test]
    fn pl_is() {
        let vars = json!({ "header": true });

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: "<p pl-is='header ? \"h1\" : \"h2\"'>this</p>".into(),
            },
        );
        assert_eq!(result, "<h1>this</h1>");
    }

    #[test]
    fn pl_html() {
        let vars = json!({ "content": "<p>hello world</p>" });

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: "<article pl-html='content'>something that used to be here</article>".into(),
            },
        );
        assert_eq!(result, "<article><p>hello world</p></article>");
    }

    #[test]
    fn pl_html_with_vars_are_not_rendered() {
        let vars = json!({ "content": "<p>hello {{mistake}} world</p>" });

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: "<article pl-html='content'>something that used to be here</article>".into(),
            },
        );
        assert_eq!(result, "<article><p>hello {{mistake}} world</p></article>");
    }

    #[test]
    fn pl_html_with_template() {
        let vars = json!({ "content": "<p>hello world</p>" });

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: "<template pl-html='content'>something that used to be here</template>"
                    .into(),
            },
        );
        assert_eq!(result, "<p>hello world</p>");
    }

    #[test]
    fn template_preserved() {
        let vars = Map::new().into();

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: "<template><h1>hello</h1></template>".into(),
            },
        );
        assert_eq!(result, "<template><h1>hello</h1></template>");
    }

    #[test]
    fn pl_for() {
        let vars = Map::new().into();

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: "<div><p pl-for='x in [1,2,3]'>{{x}}</p></div>".into(),
            },
        );
        assert_eq!(result, "<div><p>1</p><p>2</p><p>3</p></div>");
    }

    #[test]
    #[ignore]
    fn pl_for_template() {
        let vars = Map::new().into();

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: "<div><template pl-for='x in [1,2,3]'><p>{{x}}</p></template></div>".into(),
            },
        );
        assert_eq!(result, "<div><p>1</p><p>2</p><p>3</p></div>");
    }

    #[test]
    fn pl_if_template() {
        let vars = Map::new().into();

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: "<div><template pl-if='[1]'><p>hello</p><p>world</p></template></div>".into(),
            },
        );
        assert_eq!(result, "<div><p>hello</p><p>world</p></div>");
    }

    #[test]
    #[ignore]
    fn loop_with_if_else() {
        let vars = Map::new().into();

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: r#"<div pl-if='"A" == "Z"'>A</div>\
    <div pl-for="_ in [1,3]" v-else-if='"A" == "A"'>B</div>\
    <div pl-else-if='"A" == "A"'>C</div>\
    <div pl-else>Not A/B/C</div>"#
                    .into(),
            },
        );
        assert_eq!(result, "<div>B</div><div>B</div>");
    }

    #[test]
    fn pl_else_true() {
        let vars = Map::new().into();

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: r#"<p pl-if="true">A</p><p pl-else>B</p>"#.into(),
            },
        );
        assert_eq!(result, "<p>A</p>");
    }

    #[test]
    fn pl_else_false() {
        let vars = Map::new().into();

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: r#"<p pl-if="false">A</p><p pl-else>B</p>"#.into(),
            },
        );
        assert_eq!(result, "<p>B</p>");
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
        assert_eq!(result, "<article><p>hello world</p></article>");
    }

    #[test]
    fn caret_attr_eval() {
        let vars = Map::new().into();

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: r#"<input ^value='"my" + " " + "name"'>"#.into(),
            },
        );
        assert_eq!(result, "<input value='my name'>");
    }

    #[test]
    fn correct_escaping() {
        let vars = json!({"x": "<code>&lt;TAG&gt;</code>"});

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: r#"<slot pl-html="x"></slot>"#.into(),
            },
        );
        assert_eq!(result, "<slot><code>&lt;TAG&gt;</code></slot>");
    }

    #[test]
    fn caret_attr_false() {
        let vars = Map::new().into();

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: r#"<input ^disabled='false'>"#.into(),
            },
        );
        assert_eq!(result, "<input>");
    }

    #[test]
    fn caret_attr_array() {
        let vars = Map::new().into();

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: r#"<button ^class='["warn", "error"]'></button>"#.into(),
            },
        );
        assert_eq!(result, "<button class='warn error'></button>");
    }

    #[test]
    fn caret_attr_object() {
        let vars = json!({ "classes": { "should-have": true, "should-not-have": null, "should-also-have": 1 } });

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: r#"<button ^class='classes'></button>"#.into(),
            },
        );
        assert_eq!(
            result,
            "<button class='should-also-have should-have'></button>"
        );
    }

    #[test]
    fn comments_uneffected() {
        let vars = Map::new().into();

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: r#"<!-- MAKE ART NOT SOFTWARE -->"#.into(),
            },
        );
        assert_eq!(result, "<!-- MAKE ART NOT SOFTWARE -->");
    }

    #[test]
    fn order_unchanged() {
        let vars = Map::new().into();

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: r#"<meta ^disabled="false" name="x" ^content='"y"'>"#.into(),
            },
        );
        assert_eq!(result, "<meta name='x' content='y'>");
    }
}
