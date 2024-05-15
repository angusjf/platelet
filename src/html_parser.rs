use crate::deno_dom;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Node {
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
impl Node {
    pub(crate) fn to_string(&self) -> String {
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

pub(crate) fn parse_html(html: String) -> Node {
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
