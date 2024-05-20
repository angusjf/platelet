use std::fmt::Write as _;

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

fn html_text_safe(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn push_attr_value(s: &mut String, value: &String) {
    // prefer single quotes - unless it allows us to avoid escaping
    match (value.contains('\''), value.contains('"')) {
        (true, true) => {
            s.push('\'');
            s.push_str(&value.replace("'", "&#39;"));
        }
        (true, false) => {
            s.push('"');
            s.push_str(&value);
        }
        (false, true) => {
            s.push('\'');
            s.push_str(&value);
        }
        (false, false) => {
            s.push('\'');
            s.push_str(&value);
        }
    };
}

fn push_node_as_string(s: &mut String, n: &Node) {
    match n {
        Node::Doctype { doctype } => {
            s.push_str("<!DOCTYPE ");
            s.push_str(&html_text_safe(doctype));
            s.push('>')
        }
        Node::Comment { content } => write!(s, "<!--{}-->", html_text_safe(content)).unwrap(),
        Node::Text { content } => s.push_str(&html_text_safe(content)),
        Node::Element {
            name,
            attrs,
            children,
        } => {
            s.push('<');
            s.push_str(name);

            for (key, value) in attrs {
                s.push(' ');
                s.push_str(key);
                s.push('=');
                push_attr_value(s, value);
            }

            match name.as_str() {
                "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link"
                | "meta" | "param" | "source" | "track" | "wbr" => {
                    s.push('>');
                }
                _ => {
                    s.push('>');
                    for child in children {
                        push_node_as_string(s, child);
                    }
                    s.push('<');
                    s.push('/');
                    s.push_str(name);
                    s.push('>');
                }
            }
        }
        Node::Document { children } => {
            for child in children {
                push_node_as_string(s, child);
            }
        }
    }
}

impl Node {
    pub(crate) fn to_string(&self) -> String {
        let mut buf = String::with_capacity(0);
        push_node_as_string(&mut buf, self);
        buf
    }
}
