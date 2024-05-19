use core::fmt;
use regex::Regex;
use serde_json::{Map, Value};
use std::borrow::BorrowMut;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::expression_eval::{eval, truthy, EvalError};
use crate::expression_parser::expr;
use crate::for_loop_parser::for_loop;
use crate::html::Node;
use crate::html_parser::parse_html;
use crate::text_node::render_text_node;
use crate::{for_loop_runner, text_node};

pub trait Filesystem {
    fn move_to(&self, current: &String, path: &String) -> String;
    fn read(&self, file: &String) -> String;
}

enum PostRenderOperation {
    Nothing,
    ReplaceMeWith(Vec<Node>),
}

#[derive(Debug, PartialEq)]
pub enum RenderError {
    IllegalDirective(String),
    TextRender(text_node::RenderError),
    Parser,
    Eval(EvalError),
    ForLoopParser(String),
    ForLoopEval(String),
    UndefinedSlot(String),
    BadPlIsName(String),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::IllegalDirective(e) => write!(f, "ILLEGAL DIRECTIVE: {}", e),
            RenderError::TextRender(e) => write!(f, "TEXT RENDER ERROR: {:?}", e),
            RenderError::Parser => write!(f, "PARSER ERROR: {:?}", '?'),
            RenderError::Eval(e) => write!(f, "EVAL ERROR: {:?}", e),
            RenderError::ForLoopParser(e) => write!(f, "FOR LOOP PARSER ERROR:\n{}", e),
            RenderError::ForLoopEval(e) => write!(f, "FOR LOOP EVALUATION ERROR: {}", e),
            RenderError::UndefinedSlot(e) => write!(f, "UNDEFINED SLOT: {:?}", e),
            RenderError::BadPlIsName(e) => write!(f, "UNDEFINED `pl-is` NAME: {:?}", e),
        }
    }
}

fn render_elem<FS>(
    node: &mut Node,
    vars: &Value,
    slots: Rc<HashMap<String, Vec<Node>>>,
    already_included: &mut HashSet<(String, String)>,
    previous_conditional: &Option<bool>,
    next_neighbour_conditional: &mut Option<bool>,
    filename: &String,
    filesystem: &FS,
) -> Result<PostRenderOperation, RenderError>
where
    FS: Filesystem,
{
    match node {
        Node::Doctype { .. } => return Ok(PostRenderOperation::Nothing),
        Node::Document { children } => {
            render_children(
                children,
                &[vars],
                slots,
                already_included,
                filename,
                filesystem,
            )?;
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

                let fl = for_loop(&mut fl.as_ref()).map_err(RenderError::ForLoopParser)?;
                let contexts = for_loop_runner::for_loop_runner(&fl, vars)
                    .map_err(RenderError::ForLoopEval)?;
                attrs_list.remove(fl_index);

                let mut repeats = vec![];

                if name == "template" {
                    for _ in &contexts {
                        for child in children.clone() {
                            repeats.push(child.clone());
                        }
                    }
                } else {
                    for _ in &contexts {
                        repeats.push(node.clone());
                    }
                }

                render_children(
                    &mut repeats,
                    &contexts.iter().collect::<Vec<_>>(),
                    slots.clone(),
                    already_included,
                    filename,
                    filesystem,
                )?;
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

                let path = filesystem.move_to(filename, src);

                let mut slots: HashMap<_, Vec<Node>> = HashMap::new();

                let mut children = children.to_owned();

                children.retain(|item| match item {
                    Node::Element {
                        name,
                        attrs,
                        children,
                    } if name == "template" => {
                        let (_, slot_name) = attrs.iter().find(|(k, _)| k == "pl-slot").unwrap();
                        slots.insert(slot_name.clone(), children.to_owned());
                        false
                    }
                    _ => true,
                });

                slots.insert("".to_owned(), children);

                let mut new_context = Map::new();

                for (attr, val) in attrs_list.to_owned() {
                    if let Some(attr) = attr.strip_prefix("^") {
                        let exp = expr(&mut val.as_ref()).map_err(|_| RenderError::Parser)?;
                        let v = eval(&exp, vars).map_err(RenderError::Eval)?;
                        new_context.insert(attr.to_string(), v);
                    }
                }

                let rendered = render(
                    &Value::Object(new_context),
                    Rc::new(slots),
                    already_included,
                    &path,
                    filesystem,
                )?;

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
                    None => return Err(RenderError::UndefinedSlot(src.clone())),
                }
            }

            if let Some(exp_index) = attrs_list.iter().position(|(name, _)| name == "pl-is") {
                let (_, exp) = &attrs_list[exp_index];

                let exp = expr(&mut exp.as_ref()).map_err(|_| RenderError::Parser)?;
                let v = eval(&exp, vars).map_err(RenderError::Eval)?;
                match v {
                    Value::String(tag) => {
                        let html_tag_re = Regex::new(r"^(?i)[A-Z][\w.-]*$").unwrap();
                        if html_tag_re.is_match(&tag) {
                            attrs_list.remove(exp_index);
                            *name = tag;
                        } else {
                            return Err(RenderError::BadPlIsName(tag));
                        };
                    }
                    _v => {
                        return Err(RenderError::IllegalDirective(
                            "pl-is expects a string".into(),
                        ))
                    }
                }
            }

            modify_attrs(attrs_list, vars)?;

            render_children(
                children,
                &[vars],
                slots,
                already_included,
                filename,
                filesystem,
            )?;

            if name == "style" || name == "script" {
                if let [Node::Text { content }] = &children[..] {
                    let key = (
                        content.to_owned(),
                        name.to_owned()
                            + &attrs_list
                                .iter()
                                .map(|(k, v)| format!("|{}={}|", k, v))
                                .collect::<Vec<_>>()
                                .join("///"),
                    );
                    if already_included.contains(&key) {
                        return Ok(PostRenderOperation::ReplaceMeWith(vec![]));
                    } else {
                        already_included.insert(key);
                    }
                }
            }

            return Ok(PostRenderOperation::Nothing);
        }
    }
}

fn render_children<FS>(
    children: &mut Vec<Node>,
    vars: &[&Value],
    slots: Rc<HashMap<String, Vec<Node>>>,
    already_included_styles: &mut HashSet<(String, String)>,
    filename: &String,
    filesystem: &FS,
) -> Result<(), RenderError>
where
    FS: Filesystem,
{
    let mut i = 0;

    let mut get_this = None;
    let mut set_this = None;
    while i < children.len() {
        let child = children[i].borrow_mut();
        match render_elem(
            child,
            vars[i.min(vars.len() - 1)],
            slots.clone(),
            already_included_styles,
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

pub(crate) fn render<FS>(
    vars: &Value,
    slots: Rc<HashMap<String, Vec<Node>>>,
    already_included_styles: &mut HashSet<(String, String)>,
    filename: &String,
    filesystem: &FS,
) -> Result<Node, RenderError>
where
    FS: Filesystem,
{
    let html = filesystem.read(filename);

    let mut node = parse_html(html);

    let _ = render_elem(
        &mut node,
        vars,
        slots,
        already_included_styles,
        &None,
        &mut None,
        filename,
        filesystem,
    )?;

    Ok(node)
}
