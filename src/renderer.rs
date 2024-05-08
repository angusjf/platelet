use lol_html::html_content::ContentType;
use lol_html::{element, text, HtmlRewriter, Settings};
use serde_json::Value;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};

use crate::text_node::render_text_node;

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
    fn get_data_at_path(&self, path: &PathBuf) -> Vec<u8>;
}

pub fn render<F>(props: &Value, filename: &PathBuf, filesystem: &F) -> String
where
    F: Filesystem,
{
    let mut out = Vec::new();
    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                element!("*", |el| {
                    let mut replace_with = None;
                    for attr in el.attributes() {
                        let name = attr.name();
                        if !name.starts_with("pl-") {
                            continue;
                        }
                        match name.as_str() {
                            "pl-src" => {
                                replace_with = Some(match replace_with {
                                    None => Replacement::Template(SrcAndData::JustSrc(attr.value())),
                                    Some(Replacement::Template(SrcAndData::JustData(data))) => Replacement::Template(SrcAndData::BothSrcAndData(attr.value(), data)),
                                    _ => panic!("you can't use any other `pl-` tags with `pl-src`, excluding `pl-data`")
                                });
                            }
                            "pl-for" => {}
                            "pl-data" => {
                                println!("{}", attr.value());
                                let data = serde_json::from_str(&attr.value()).unwrap();
                                replace_with = Some(match replace_with {
                                    None => Replacement::Template(SrcAndData::JustData(data)),
                                    Some(Replacement::Template(SrcAndData::JustSrc(src))) => Replacement::Template(SrcAndData::BothSrcAndData(src, data)),
                                    _ => panic!("you can't use any other `pl-` tags with `pl-src`, excluding `pl-data`")
                                });
                            }
                            "pl-outer-html" => {}
                            _ => {
                                eprintln!("unexpected `pl-` attribute `{}`", name);
                            }
                        }
                    }
                    match replace_with {
                        Some(Replacement::Template(src_data)) => {
                            let (src, data) = match src_data {
                                SrcAndData::BothSrcAndData(src, data) => (src, data),
                                SrcAndData::JustSrc(src) => (src, Value::Null),
                                _ => panic!("bad or missing pl-src"),
                            };
                            let path = filename.parent().unwrap().join(src);
                            let rendered = render(&data, &path, filesystem);
                            el.replace(&rendered, ContentType::Html)
                        }
                        None => {}
                    }
                    Ok(())
                }),
                text!("*", |node| {
                    let txt = node.as_str();
                    match render_text_node(txt, &props) {
                        Ok(content) => {
                            let content = content.to_string();
                            node.replace(content.as_ref(), ContentType::Text);
                        }
                        Err(e) => panic!("{:?}", e),
                    }
                    Ok(())
                }),
            ],
            ..Settings::default()
        },
        |c: &[u8]| out.extend_from_slice(c),
    );

    let data = filesystem.get_data_at_path(&filename);

    rewriter.write(&data).expect("can't write to rewriter");

    rewriter.end().expect("no end");

    String::from_utf8(out).unwrap()
}

#[cfg(test)]
mod test {

    use serde_json::json;

    use super::*;

    struct MockFilesystem {
        data: Vec<u8>,
    }

    impl Filesystem for MockFilesystem {
        fn get_data_at_path(&self, _: &PathBuf) -> Vec<u8> {
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
                data: b"<h1>nothing here</h1>".into(),
            },
        );
        assert_eq!(result, "<h1>nothing here</h1>");
    }

    #[test]
    fn templated_text_node() {
        let vars = json!({ "hello": "world" });

        let result = render(
            &vars,
            &PathBuf::new(),
            &MockFilesystem {
                data: b"<h1>{{hello}}</h1>".into(),
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
                data: b"<h1>Dear {{user.firstname}} {{user.lastname}},</h1>".into(),
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
                data: b"<h1>{{countries[0]}} {{ 1 + 2 }}</h1>".into(),
            },
        );
        assert_eq!(result, "<h1>portugal 3</h1>");
    }
}
