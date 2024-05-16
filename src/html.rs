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
