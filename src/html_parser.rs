use crate::html::Node as N;
use crate::rcdom::Node;
use crate::rcdom::NodeData;
pub use crate::rcdom::RcDom;
use html5ever::driver::parse_document;
use html5ever::driver::parse_fragment;
use html5ever::driver::ParseOpts;
use html5ever::tendril::stream::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{namespace_url, ns};
use markup5ever::Attribute;
use markup5ever::{LocalName, QualName};
use std::cell::RefCell;
use std::rc::Rc;

/// credit to the deno-dom dev, https://github.com/b-fuze
/// MIT Licence

pub(crate) fn parse_html(html: String) -> N {
    let full_doc = {
        let html = html.to_lowercase();
        html.contains("<html")
            || html.contains("<body")
            || html.contains("<head")
            || html.contains("<!DOCTYPE")
    };

    if full_doc {
        parse(html)
    } else {
        match parse_frag(html, "body".to_owned()) {
            N::Document { ref children } => match &children[..] {
                [N::Element {
                    ref name,
                    ref attrs,
                    children,
                }] if name == "html" && attrs.is_empty() => N::Document {
                    children: children.clone(),
                },
                _ => panic!("!!!!!!!!"),
            },
            _ => panic!("#"),
        }
    }
}

fn parse(html: String) -> N {
    let sink: RcDom = Default::default();
    let parser = parse_document(
        sink,
        ParseOpts {
            tokenizer: Default::default(),
            tree_builder: TreeBuilderOpts {
                scripting_enabled: false,
                ..Default::default()
            },
        },
    );

    let dom = parser.one(html);
    nodeify_node(&dom.document)
}

fn parse_frag(html: String, context_local_name: String) -> N {
    let sink: RcDom = Default::default();
    let parser = parse_fragment(
        sink,
        Default::default(),
        QualName::new(None, ns!(html), LocalName::from(context_local_name)),
        vec![],
    );

    let dom = parser.one(html);
    nodeify_node(&dom.document)
}

fn nodeify_node(dom: &Rc<Node>) -> N {
    match dom.data {
        NodeData::Document => {
            let mut children = vec![];
            for c in dom.children.borrow().iter() {
                children.push(nodeify_node(c));
            }
            N::Document { children }
        }

        NodeData::Element {
            ref name,
            ref attrs,
            ref template_contents,
            ..
        } => {
            let name = name.local.to_string();

            let attrs = nodify_attrs(attrs);

            let mut children = vec![];

            if let Some(contents) = template_contents {
                children.push(nodeify_node(&contents));
            } else {
                for child in dom.children.borrow().iter() {
                    children.push(nodeify_node(&child));
                }
            }

            N::Element {
                name,
                attrs,
                children,
            }
        }

        NodeData::Text { ref contents } => N::Text {
            content: contents.borrow().to_string(),
        },

        NodeData::Comment { ref contents } => N::Comment {
            content: contents.to_string(),
        },

        NodeData::Doctype {
            ref name,
            public_id: _,
            system_id: _,
        } => N::Doctype {
            doctype: name.to_string(),
        },

        _ => {
            panic!();
        }
    }
}

fn nodify_attrs(data: &RefCell<Vec<Attribute>>) -> Vec<(String, String)> {
    let mut attrs = vec![];

    for attr in data.borrow().iter() {
        attrs.push((attr.name.local.to_string(), attr.value.to_string()));
    }

    attrs
}
