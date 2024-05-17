use core::fmt;
use serde_json::Value;
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use crate::expression_eval::{eval, truthy, EvalError};
use crate::expression_parser::expr;
use crate::for_loop_parser::for_loop;
use crate::html::Node;
use crate::html_parser::parse_html;
use crate::text_node::render_text_node;
use crate::{for_loop_runner, text_node};

pub trait Filesystem {
    fn get_data_at_path(&self, path: &PathBuf) -> String;
}

enum PostRenderOperation {
    Nothing,
    ReplaceMeWith(Vec<Node>),
}

#[derive(Debug)]
pub enum RenderError {
    IllegalDirective(String),
    TextRender(text_node::RenderError),
    Parser,
    ForLoopParser(()),
    Eval(EvalError),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::IllegalDirective(e) => write!(f, "ILLEGAL DIRECTIVE: {}", e),
            RenderError::TextRender(e) => write!(f, "TEXT RENDER ERROR: {:?}", e),
            RenderError::Parser => write!(f, "PARSER ERROR: {:?}", '?'),
            RenderError::Eval(e) => write!(f, "EVAL ERROR: {:?}", e),
            RenderError::ForLoopParser(e) => write!(f, "FOR LOOP PARSER: {:?}", e),
        }
    }
}

fn render_elem<F>(
    node: &mut Node,
    vars: &Value,
    slots: Rc<HashMap<String, Vec<Node>>>,
    previous_conditional: &Option<bool>,
    next_neighbour_conditional: &mut Option<bool>,
    filename: &PathBuf,
    filesystem: &F,
) -> Result<PostRenderOperation, RenderError>
where
    F: Filesystem,
{
    match node {
        Node::Doctype { .. } => return Ok(PostRenderOperation::Nothing),
        Node::Document { children } => {
            render_children(children, vars, slots, filename, filesystem)?;
            return Ok(PostRenderOperation::Nothing);
        }
        Node::Comment { .. } => return Ok(PostRenderOperation::Nothing),
        Node::Text { content: t, .. } => {
            let content = render_text_node(t.as_ref(), &vars).map_err(RenderError::TextRender)?;
            let content = content.to_string();
            *t = content;
            Ok(PostRenderOperation::Nothing)
        }
        Node::Element {
            attrs: attrs_list,
            children,
            name,
            ..
        } => {
            if let Some(exp_index) = attrs_list.iter().position(|(name, _)| name == "pl-if") {
                let (_, exp) = &attrs_list[exp_index];
                let exp = expr(&mut exp.as_ref()).map_err(|_| RenderError::Parser)?;
                let v = eval(&exp, vars).map_err(RenderError::Eval)?;
                let cond = truthy(&v);
                *next_neighbour_conditional = Some(cond);
                if !cond {
                    return Ok(PostRenderOperation::ReplaceMeWith(vec![]));
                } else if name == "template" {
                    return Ok(PostRenderOperation::ReplaceMeWith(children.to_owned()));
                }
                attrs_list.remove(exp_index);
            }

            if let Some(exp_index) = attrs_list.iter().position(|(name, _)| name == "pl-else-if") {
                let (_, exp) = &attrs_list[exp_index];
                match previous_conditional {
                    Some(true) => {
                        *next_neighbour_conditional = Some(true);
                        return Ok(PostRenderOperation::ReplaceMeWith(vec![]));
                    }
                    Some(false) => {
                        let exp = expr(&mut exp.as_ref()).map_err(|_| RenderError::Parser)?;
                        let v = eval(&exp, vars).map_err(RenderError::Eval)?;
                        let cond = truthy(&v);
                        *next_neighbour_conditional = Some(cond);
                        if !cond {
                            return Ok(PostRenderOperation::ReplaceMeWith(vec![]));
                        }
                        attrs_list.remove(exp_index);
                    }
                    None => {
                        return Err(RenderError::IllegalDirective(
                            "encountered a pl-else-if that didn't follow an if".into(),
                        ))
                    }
                }
            }

            if let Some(index) = attrs_list.iter().position(|(name, _)| name == "pl-else") {
                match previous_conditional {
                    Some(true) => {
                        return Ok(PostRenderOperation::ReplaceMeWith(vec![]));
                    }
                    Some(false) => {
                        attrs_list.remove(index);
                    }
                    None => return Err(RenderError::IllegalDirective(
                        "encountered a pl-else that didn't immediately for a pl-if or pl-else-if"
                            .into(),
                    )),
                }
            }

            if let Some(fl_index) = attrs_list.iter().position(|(name, _)| name == "pl-for") {
                let (_, fl) = &attrs_list[fl_index];

                let fl = for_loop(&mut fl.as_ref()).map_err(|_| RenderError::ForLoopParser(()))?;
                let contexts = for_loop_runner::for_loop_runner(&fl, vars).unwrap();
                attrs_list.remove(fl_index);

                let mut repeats = vec![];
                if name == "template" {
                    for ctx in contexts {
                        for child in children.clone() {
                            let mut child = child.clone();
                            render_elem(
                                &mut child,
                                &ctx,
                                slots.clone(),
                                &None,
                                &mut None,
                                filename,
                                filesystem,
                            )?;
                            repeats.push(child);
                        }
                    }
                } else {
                    for ctx in contexts {
                        let mut copy = node.clone();
                        render_elem(
                            &mut copy,
                            &ctx,
                            slots.clone(),
                            &None,
                            &mut None,
                            filename,
                            filesystem,
                        )?;
                        repeats.push(copy);
                    }
                }
                return Ok(PostRenderOperation::ReplaceMeWith(repeats));
            }

            if let Some(exp_index) = attrs_list.iter().position(|(name, _)| name == "pl-html") {
                let (_, exp) = &attrs_list[exp_index];

                let exp = expr(&mut exp.as_ref()).map_err(|_| RenderError::Parser)?;
                let exp = eval(&exp, vars).map_err(RenderError::Eval)?;
                match exp {
                    Value::String(html) => {
                        let node = parse_html(html);
                        if name == "template" {
                            return Ok(PostRenderOperation::ReplaceMeWith(vec![node]));
                        } else {
                            attrs_list.remove(exp_index);
                            children.clear();
                            children.push(node);
                            return Ok(PostRenderOperation::Nothing);
                        }
                    }
                    _v => {
                        return Err(RenderError::IllegalDirective(
                            "pl-html expects a string".into(),
                        ))
                    }
                }
            }

            if let Some(src_index) = attrs_list.iter().position(|(name, _)| name == "pl-src") {
                let (_, src) = &attrs_list[src_index];

                let path = filename.parent().unwrap().join(src);

                let slots: HashMap<_, Vec<Node>> =
                    HashMap::from([("".to_owned(), children.clone())]);

                let rendered = render(vars, Rc::new(slots), &path, filesystem)?;
                match rendered {
                    Node::Document { children } => {
                        return Ok(PostRenderOperation::ReplaceMeWith(children))
                    }
                    _ => panic!("I know that render only ever returns a document"),
                }
            }

            if let Some(src_index) = attrs_list.iter().position(|(name, _)| name == "pl-slot") {
                let (_, src) = &attrs_list[src_index];

                match slots.get(src) {
                    Some(node) => return Ok(PostRenderOperation::ReplaceMeWith(node.clone())),
                    None => panic!("no slot provided"),
                }
            }

            if let Some(exp_index) = attrs_list.iter().position(|(name, _)| name == "pl-is") {
                let (_, exp) = &attrs_list[exp_index];

                let exp = expr(&mut exp.as_ref()).map_err(|_| RenderError::Parser)?;
                let v = eval(&exp, vars).map_err(RenderError::Eval)?;
                match v {
                    Value::String(tag) => {
                        attrs_list.remove(exp_index);
                        *name = tag;
                    }
                    _v => {
                        return Err(RenderError::IllegalDirective(
                            "pl-is expects a string".into(),
                        ))
                    }
                }
            }

            modify_attrs(attrs_list, vars)?;

            render_children(children, vars, slots, filename, filesystem)?;

            return Ok(PostRenderOperation::Nothing);
        }
    }
}

fn render_children<F>(
    children: &mut Vec<Node>,
    vars: &Value,
    slots: Rc<HashMap<String, Vec<Node>>>,
    filename: &PathBuf,
    filesystem: &F,
) -> Result<(), RenderError>
where
    F: Filesystem,
{
    let mut i = 0;

    let mut get_this = None;
    let mut set_this = None;
    while i < children.len() {
        let child = children[i].borrow_mut();
        match render_elem(
            child,
            vars,
            slots.clone(),
            &get_this,
            &mut set_this,
            filename,
            filesystem,
        )? {
            PostRenderOperation::Nothing => {
                i += 1;
            }
            PostRenderOperation::ReplaceMeWith(nodes) => {
                let _ = children.remove(i);
                for node in nodes {
                    children.insert(i, node);
                    i += 1;
                }
            }
        };
        get_this = set_this;
    }

    Ok(())
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

            if xs.is_empty() {
                None
            } else {
                Some(xs.join(" "))
            }
        }
        Value::Object(o) => {
            let xs: Vec<_> = o
                .iter()
                .filter_map(|(k, v)| if truthy(v) { Some(k.to_owned()) } else { None })
                .collect();

            if xs.is_empty() {
                None
            } else {
                Some(xs.join(" "))
            }
        }
    }
}

fn modify_attrs(attrs: &mut Vec<(String, String)>, vars: &Value) -> Result<(), RenderError> {
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

    Ok(())
}

fn render<F>(
    vars: &Value,
    slots: Rc<HashMap<String, Vec<Node>>>,
    filename: &PathBuf,
    filesystem: &F,
) -> Result<Node, RenderError>
where
    F: Filesystem,
{
    let html = filesystem.get_data_at_path(filename);

    let mut node = parse_html(html);

    render_elem(
        &mut node, vars, slots, &None, &mut None, filename, filesystem,
    )?;

    Ok(node)
}

pub fn render_to_string<F>(
    vars: &Value,
    filename: &PathBuf,
    filesystem: &F,
) -> Result<String, RenderError>
where
    F: Filesystem,
{
    render(vars, Rc::new(HashMap::new()), filename, filesystem).map(|x| x.to_string())
}

pub fn render_string(vars: &Value, html: String) -> Result<String, RenderError> {
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
        assert_eq!(result.unwrap(), "<h1>nothing here</h1>");
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
            result.unwrap(),
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
        assert_eq!(result.unwrap(), "<h1>world</h1>");
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
        assert_eq!(result.unwrap(), "<h1>Dear Yuri Gagarin,</h1>");
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
        assert_eq!(result.unwrap(), "<h1>portugal 3</h1>");
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
        assert_eq!(result.unwrap(), "<p>this</p><p>also this</p>");
    }

    #[test]
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
        assert_eq!(result.unwrap(), "<p>this</p><p>also this</p>");
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
        assert_eq!(result.unwrap(), "<p>this</p>");
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
        assert_eq!(result.unwrap(), "<h1>this</h1>");
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
        assert_eq!(result.unwrap(), "<article><p>hello world</p></article>");
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
        assert_eq!(
            result.unwrap(),
            "<article><p>hello {{mistake}} world</p></article>"
        );
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
        assert_eq!(result.unwrap(), "<p>hello world</p>");
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
        assert_eq!(result.unwrap(), "<template><h1>hello</h1></template>");
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
        assert_eq!(result.unwrap(), "<div><p>1</p><p>2</p><p>3</p></div>");
    }

    #[test]
    fn pl_for_template() {
        let vars = Map::new().into();

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: "<div><template pl-for='x in [1,2,3]'><p>{{x}}</p></template></div>".into(),
            },
        );
        assert_eq!(result.unwrap(), "<div><p>1</p><p>2</p><p>3</p></div>");
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
        assert_eq!(result.unwrap(), "<div><p>hello</p><p>world</p></div>");
    }

    #[test]
    fn loop_with_if_else() {
        let vars = Map::new().into();

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: "<div pl-if='\"A\" == \"Z\"'>A</div>\
                         <div pl-for='_ in [1,3]' pl-else-if='\"A\" == \"A\"'>B</div>\
                         <div pl-else-if='\"A\" == \"A\"'>C</div>\
                         <div pl-else>Not A/B/C</div>"
                    .into(),
            },
        );
        assert_eq!(result.unwrap(), "<div>B</div><div>B</div>");
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
        assert_eq!(result.unwrap(), "<p>A</p>");
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
        assert_eq!(result.unwrap(), "<p>B</p>");
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
    fn caret_attr_eval() {
        let vars = Map::new().into();

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: r#"<input ^value='"my" + " " + "name"'>"#.into(),
            },
        );
        assert_eq!(result.unwrap(), "<input value='my name'>");
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
        assert_eq!(result.unwrap(), "<slot><code>&lt;TAG&gt;</code></slot>");
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
        assert_eq!(result.unwrap(), "<input>");
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
        assert_eq!(result.unwrap(), "<button class='warn error'></button>");
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
            result.unwrap(),
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
        assert_eq!(result.unwrap(), "<!-- MAKE ART NOT SOFTWARE -->");
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
        assert_eq!(result.unwrap(), "<meta name='x' content='y'>");
    }

    #[test]
    fn for_loop_array_index() {
        let vars = Map::new().into();

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: r#"<hr pl-for="(x, i) in [1,2,3]" ^name="x" ^class="i">"#.into(),
            },
        );
        assert_eq!(
            result.unwrap(),
            "<hr name='1' class='0'><hr name='2' class='1'><hr name='3' class='2'>"
        );
    }

    #[test]
    fn for_loop_kv() {
        let vars = json!({"fields": {"first-name": "First Name", "last-name": "Last Name"}});

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: r#"<input pl-for="(k, v) in fields" ^name="k" ^placeholder="v">"#.into(),
            },
        );
        assert_eq!(
            result.unwrap(),
            "<input name='first-name' placeholder='First Name'>\
             <input name='last-name' placeholder='Last Name'>"
        );
    }

    #[test]
    fn for_loop_kvi() {
        let vars = json!({"fields": {"first-name": "First Name", "last-name": "Last Name"}});

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data:
                    r#"<input pl-for="(k, v, i) in fields" ^name="k + '-' + i" ^placeholder="v">"#
                        .into(),
            },
        );
        assert_eq!(
            result.unwrap(),
            "<input name='first-name-0' placeholder='First Name'>\
             <input name='last-name-1' placeholder='Last Name'>"
        );
    }

    #[test]
    fn for_loop_if_else_if() {
        let vars = Map::new().into();

        let result = render_to_string(
            &vars,
            &PathBuf::new(),
            &MockSingleFile {
                data: "<div pl-if='false'>A</div>\
                      <div pl-for='x in [1,2,3]' pl-else-if='true'>B</div>\
                      <div>C</div>"
                    .into(),
            },
        );
        assert_eq!(
            result.unwrap(),
            "<div>B</div><div>B</div><div>B</div><div>C</div>"
        );
    }
}
