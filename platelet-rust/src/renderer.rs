use core::fmt;
use regex::Regex;
use serde_json::{Map, Value};
use std::borrow::BorrowMut;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::rc::Rc;

use crate::expression_eval::{eval, truthy, EvalError};
use crate::expression_parser::expr;
use crate::for_loop_parser::for_loop;
use crate::html::Node;
use crate::html_parser::parse_html;
use crate::text_node::render_text_node;
use crate::types::Type;
use crate::{for_loop_runner, text_node};

pub trait Filesystem<E> {
    fn move_to(&self, current: &String, path: &String) -> Result<String, E>;
    fn read(&self, file: &String) -> Result<String, E>;
}

enum PostRenderOperation {
    Nothing,
    ReplaceMeWith(Vec<Node>),
}

#[derive(Debug, PartialEq)]
pub enum RenderErrorKind<FilesystemError> {
    IllegalDirective(String),
    TextRender(text_node::RenderError),
    Parser,
    Eval(EvalError),
    ForLoopParser(String),
    ForLoopEval(for_loop_runner::Error),
    UndefinedSlot(String),
    BadPlIsName(String),
    FilesystemError(FilesystemError),
}

impl<FilesystemError> fmt::Display for RenderErrorKind<FilesystemError>
where
    FilesystemError: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderErrorKind::IllegalDirective(e) => write!(f, "ILLEGAL DIRECTIVE: {}", e),
            RenderErrorKind::TextRender(e) => write!(f, "TEXT RENDER ERROR: {:?}", e),
            RenderErrorKind::Parser => write!(f, "PARSER ERROR: {:?}", '?'),
            RenderErrorKind::Eval(e) => write!(f, "EVAL ERROR: {:?}", e),
            RenderErrorKind::ForLoopParser(e) => write!(f, "FOR LOOP PARSER ERROR:\n{}", e),
            RenderErrorKind::ForLoopEval(e) => {
                write!(f, "FOR LOOP EVALUATION ERROR: ")?;
                match e {
                    for_loop_runner::Error::TypeMismatch { expected, found } => {
                        write!(
                            f,
                            "Expected {}, found {}",
                            expected
                                .iter()
                                .map(Type::to_string)
                                .collect::<Vec<_>>()
                                .join(" or "),
                            found.to_string()
                        )
                    }

                    for_loop_runner::Error::Eval(e) => write!(f, "{:?}", e),
                }
            }
            RenderErrorKind::UndefinedSlot(e) => write!(f, "UNDEFINED SLOT: {:?}", e),
            RenderErrorKind::BadPlIsName(e) => write!(f, "UNDEFINED `pl-is` NAME: {:?}", e),
            RenderErrorKind::FilesystemError(e) => write!(f, "FILE SYSTEM ERROR: {:?}", e),
        }
    }
}

fn parse_eval<T>(exp: &String, vars: &Value) -> Result<Value, RenderErrorKind<T>> {
    let exp = expr(&mut exp.as_ref()).map_err(|_| RenderErrorKind::Parser)?;
    eval(&exp, vars).map_err(RenderErrorKind::Eval)
}

#[derive(PartialEq, Debug)]
pub struct RenderError<FilesystemError> {
    pub kind: RenderErrorKind<FilesystemError>,
    pub filename: String,
}

impl<FilesystemError> fmt::Display for RenderError<FilesystemError>
where
    FilesystemError: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.kind)?;
        write!(f, "in {}", self.filename)
    }
}

fn render_elem<FS, FilesystemError>(
    node: &mut Node,
    vars: &Value,
    slots: Rc<HashMap<String, Vec<Node>>>,
    already_included: &mut HashSet<(String, String)>,
    previous_conditional: &Option<bool>,
    next_neighbour_conditional: &mut Option<bool>,
    filename: &String,
    filesystem: &FS,
) -> Result<PostRenderOperation, RenderError<FilesystemError>>
where
    FS: Filesystem<FilesystemError>,
    FilesystemError: fmt::Debug,
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
            let content = render_text_node(t.as_ref(), &vars).map_err(|e| RenderError {
                kind: RenderErrorKind::TextRender(e),
                filename: filename.to_owned(),
            })?;
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
                let v = parse_eval(exp, vars).map_err(|e| RenderError {
                    kind: e,
                    filename: filename.to_owned(),
                })?;
                let cond = truthy(&v);
                *next_neighbour_conditional = Some(cond);
                if !cond {
                    return Ok(PostRenderOperation::ReplaceMeWith(vec![]));
                } else {
                    attrs_list.remove(exp_index);
                }
            }

            if let Some(exp_index) = attrs_list.iter().position(|(name, _)| name == "pl-else-if") {
                let (_, exp) = &attrs_list[exp_index];
                match previous_conditional {
                    Some(true) => {
                        *next_neighbour_conditional = Some(true);
                        return Ok(PostRenderOperation::ReplaceMeWith(vec![]));
                    }
                    Some(false) => {
                        let v = parse_eval(exp, vars).map_err(|e| RenderError {
                            kind: e,
                            filename: filename.clone(),
                        })?;
                        let cond = truthy(&v);
                        *next_neighbour_conditional = Some(cond);
                        if !cond {
                            return Ok(PostRenderOperation::ReplaceMeWith(vec![]));
                        } else {
                            attrs_list.remove(exp_index);
                        }
                    }
                    None => {
                        return Err(RenderError {
                            kind: RenderErrorKind::IllegalDirective(
                                "encountered a pl-else-if that didn't follow an if".into(),
                            ),
                            filename: filename.to_owned(),
                        })
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
                    None => return Err(RenderError {
                        kind: RenderErrorKind::IllegalDirective(
                        "encountered a pl-else that didn't immediately for a pl-if or pl-else-if"
                            .into(),
                    ),
                    filename: filename.to_owned(),
                }),
                }
            }

            if let Some(fl_index) = attrs_list.iter().position(|(name, _)| name == "pl-for") {
                let (_, fl) = &attrs_list[fl_index];

                let fl = for_loop(&mut fl.as_ref())
                    .map_err(RenderErrorKind::ForLoopParser)
                    .map_err(|e| RenderError {
                        kind: e,
                        filename: filename.clone(),
                    })?;
                let contexts = for_loop_runner::for_loop_runner(&fl, vars)
                    .map_err(RenderErrorKind::ForLoopEval)
                    .map_err(|e| RenderError {
                        kind: e,
                        filename: filename.clone(),
                    })?;
                attrs_list.remove(fl_index);

                let mut repeats = vec![];

                for _ in &contexts {
                    repeats.push(node.clone());
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

            if let Some(exp_index) = attrs_list.iter().position(|(name, _)| name == "pl-is") {
                let (_, exp) = &attrs_list[exp_index];

                let v = parse_eval(&exp, vars).map_err(|e| RenderError {
                    kind: e,
                    filename: filename.clone(),
                })?;
                match v {
                    Value::String(tag) => {
                        let html_tag_re = Regex::new(r"^(?i)[A-Z][\w.-]*$").unwrap();
                        if html_tag_re.is_match(&tag) {
                            attrs_list.remove(exp_index);
                            *name = tag;
                        } else {
                            return Err(RenderError {
                                kind: RenderErrorKind::BadPlIsName(tag),
                                filename: filename.to_owned(),
                            });
                        };
                    }
                    _v => {
                        return Err(RenderError {
                            kind: RenderErrorKind::IllegalDirective(
                                "pl-is expects a string".into(),
                            ),
                            filename: filename.to_owned(),
                        })
                    }
                }
            }

            if let Some(exp_index) = attrs_list.iter().position(|(name, _)| name == "pl-html") {
                let (_, exp) = &attrs_list[exp_index];

                let v = parse_eval(exp, vars).map_err(|e| RenderError {
                    kind: e,
                    filename: filename.to_owned(),
                })?;
                match v {
                    Value::String(html) => {
                        let node = parse_html(html);
                        attrs_list.remove(exp_index);
                        children.clear();
                        children.push(node);
                        return Ok(PostRenderOperation::Nothing);
                    }
                    _v => {
                        return Err(RenderError {
                            kind: RenderErrorKind::IllegalDirective(
                                "pl-html expects a string".into(),
                            ),
                            filename: filename.to_owned(),
                        })
                    }
                }
            }

            if let Some(src_index) = attrs_list.iter().position(|(name, _)| name == "pl-src") {
                let (_, src) = &attrs_list[src_index];

                let path = filesystem
                    .move_to(filename, src)
                    .map_err(RenderErrorKind::FilesystemError)
                    .map_err(|e| RenderError {
                        kind: e,
                        filename: filename.to_owned(),
                    })?;

                let mut slots: HashMap<_, Vec<Node>> = HashMap::new();

                let mut children = children.to_owned();

                children.retain(|item| match item {
                    Node::Element {
                        name,
                        attrs,
                        children,
                    } if name == "pl-template" => {
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
                        let v = parse_eval(&val, vars).map_err(|e| RenderError {
                            kind: e,
                            filename: filename.to_owned(),
                        })?;
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
                        return Ok(PostRenderOperation::ReplaceMeWith(vec![Node::Document {
                            children: children.clone(),
                        }]))
                    }
                    _ => panic!("I know that render only ever returns a document"),
                }
            }

            if let Some(src_index) = attrs_list.iter().position(|(name, _)| name == "pl-slot") {
                let (_, src) = &attrs_list[src_index];

                match slots.get(src) {
                    Some(node) => return Ok(PostRenderOperation::ReplaceMeWith(node.clone())),
                    None => {
                        return Err(RenderError {
                            kind: RenderErrorKind::UndefinedSlot(src.clone()),
                            filename: filename.to_owned(),
                        })
                    }
                }
            }

            modify_attrs(attrs_list, vars).map_err(|e| RenderError {
                kind: e,
                filename: filename.clone(),
            })?;

            if name != "script" {
                render_children(
                    children,
                    &[vars],
                    slots,
                    already_included,
                    filename,
                    filesystem,
                )?;
            } else {
                // TODO - should I allow injecting script tags?
            }

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

fn render_children<FS, FilesystemError>(
    children: &mut Vec<Node>,
    vars: &[&Value],
    slots: Rc<HashMap<String, Vec<Node>>>,
    already_included_styles: &mut HashSet<(String, String)>,
    filename: &String,
    filesystem: &FS,
) -> Result<(), RenderError<FilesystemError>>
where
    FS: Filesystem<FilesystemError>,
    FilesystemError: fmt::Debug,
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

fn modify_attrs<FileSystemError>(
    attrs: &mut Vec<(String, String)>,
    vars: &Value,
) -> Result<(), RenderErrorKind<FileSystemError>> {
    let mut ret: Result<(), RenderErrorKind<FileSystemError>> = Ok(());

    attrs.retain_mut(|(name_original, val)| {
        if let Some(name) = name_original.strip_prefix('^') {
            match parse_eval(&val, vars) {
                Ok(v) => match attrify(&v) {
                    None => false,
                    Some(s) => {
                        *name_original = name.to_owned();
                        *val = s;
                        true
                    }
                },
                Err(e) => {
                    ret = Err(e);
                    false
                }
            }
        } else {
            true
        }
    });

    ret
}

pub(crate) fn render<FS, FileSystemError>(
    vars: &Value,
    slots: Rc<HashMap<String, Vec<Node>>>,
    already_included_styles: &mut HashSet<(String, String)>,
    filename: &String,
    filesystem: &FS,
) -> Result<Node, RenderError<FileSystemError>>
where
    FS: Filesystem<FileSystemError>,
    FileSystemError: fmt::Debug,
{
    let html = filesystem.read(filename).map_err(|e| RenderError {
        kind: RenderErrorKind::FilesystemError(e),
        filename: filename.to_owned(),
    })?;

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
