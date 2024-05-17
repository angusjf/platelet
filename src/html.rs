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

impl Node {
    pub(crate) fn to_string(&self) -> String {
        fn html_text_safe(s: &str) -> String {
            s.replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
        }
        fn html_attr_safe_quoted(s: &str) -> String {
            // prefer single quotes - unless it allows us to avoid escaping
            let (outer_quotes, sanetized) = match (s.contains('\''), s.contains('"')) {
                (true, true) => ('\'', s.replace("'", "&#39;")),
                (true, false) => ('"', s.to_owned()),
                (false, true) => ('\'', s.to_owned()),
                (false, false) => ('\'', s.to_owned()),
            };

            format!("{}{}{}", outer_quotes, sanetized, outer_quotes)
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
                    .map(|(key, value)| format!(" {}={}", key, html_attr_safe_quoted(value)))
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
