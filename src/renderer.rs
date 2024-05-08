use lol_html::html_content::ContentType;
use lol_html::{element, text, HtmlRewriter, Settings};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};

use crate::expression_eval::{eval, truthy};
use crate::expression_parser::expr;
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

pub fn render<F>(vars: &Value, filename: &PathBuf, filesystem: &F) -> String
where
    F: Filesystem,
{
    let mut out = Vec::new();
    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                element!("*", |el| {
                    let pl_attrs: HashMap<_, _> = el
                        .attributes()
                        .iter()
                        .filter_map(|attr| {
                            let name = attr.name();
                            if name.starts_with("pl-") {
                                Some((name, attr.value()))
                            } else {
                                None
                            }
                        })
                        .collect();

                    pl_attrs.keys().for_each(|name| el.remove_attribute(&name));

                    if let Some(exp) = pl_attrs.get("pl-if") {
                        match expr(&mut exp.as_str()) {
                            Ok(exp) => match eval(&exp, vars) {
                                Ok(v) => {
                                    if !truthy(&v) {
                                        el.remove()
                                    }
                                }
                                Err(e) => todo!(),
                            },
                            Err(x) => todo!(),
                        }
                    }
                    if let Some(exp) = pl_attrs.get("pl-else-if") {}
                    if let Some(exp) = pl_attrs.get("pl-else") {}
                    if let Some(exp) = pl_attrs.get("pl-for") {
                        todo!()
                    }
                    if let Some(exp) = pl_attrs.get("pl-html") {
                        match expr(&mut exp.as_str()) {
                            Ok(exp) => match eval(&exp, vars) {
                                Ok(Value::String(content)) => {
                                    el.set_inner_content(&content, ContentType::Html)
                                }
                                Ok(_) => todo!(),
                                Err(e) => todo!(),
                            },
                            Err(x) => todo!(),
                        }
                    }
                    if let Some(exp) = pl_attrs.get("pl-attrs") {}
                    if let Some(exp) = pl_attrs.get("pl-src") {}
                    if let Some(exp) = pl_attrs.get("pl-data") {}
                    if let Some(exp) = pl_attrs.get("pl-slot") {}

                    if let Some(exp) = pl_attrs.get("pl-is") {
                        match expr(&mut exp.as_str()) {
                            Ok(exp) => match eval(&exp, vars) {
                                Ok(Value::String(tag_name)) => el.set_tag_name(&tag_name).unwrap(),
                                Ok(_) => todo!(),
                                Err(e) => todo!(),
                            },
                            Err(x) => todo!(),
                        }
                    }

                    Ok(())
                    // for (name, value) in pl_attrs {
                    //     match name.as_str() {
                    //         "pl-src" => {
                    //             replace_with = Some(match replace_with {
                    //                 None => Replacement::Template(SrcAndData::JustSrc(value)),
                    //                 Some(Replacement::Template(SrcAndData::JustData(data))) => Replacement::Template(SrcAndData::BothSrcAndData(value, data)),
                    //                 _ => panic!("you can't use any other `pl-` tags with `pl-src`, excluding `pl-data`")
                    //             });
                    //         }
                    //         "pl-if" => {
                    //             el.remove();
                    //         }
                    //         "pl-for" => {}
                    //         "pl-data" => {
                    //             println!("{}", value);
                    //             let data = serde_json::from_str(&value).unwrap();
                    //             replace_with = Some(match replace_with {
                    //                 None => Replacement::Template(SrcAndData::JustData(data)),
                    //                 Some(Replacement::Template(SrcAndData::JustSrc(src))) => Replacement::Template(SrcAndData::BothSrcAndData(src, data)),
                    //                 _ => panic!("you can't use any other `pl-` tags with `pl-src`, excluding `pl-data`")
                    //             });
                    //         }
                    //         "pl-outer-html" => {}
                    //         _ => {
                    //             eprintln!("unexpected `pl-` attribute `{}`", name);
                    //         }
                    //     }
                    // }
                    // match replace_with {
                    //     Some(Replacement::Template(src_data)) => {
                    //         let (src, data) = match src_data {
                    //             SrcAndData::BothSrcAndData(src, data) => (src, data),
                    //             SrcAndData::JustSrc(src) => (src, Value::Null),
                    //             _ => panic!("bad or missing pl-src"),
                    //         };
                    //         let path = filename.parent().unwrap().join(src);
                    //         let rendered = render(&data, &path, filesystem);
                    //         el.replace(&rendered, ContentType::Html)
                    //     }
                    //     None => {}
                    // }
                }),
                text!("*", |node| {
                    let txt = node.as_str();
                    match render_text_node(txt, &vars) {
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

    use serde_json::{json, Map};

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

    #[test]
    fn pl_if() {
        let vars = Map::new().into();

        let result = render(
            &vars,
            &PathBuf::new(),
            &MockFilesystem {
                data: b"<p>this</p>\
                    <p pl-if='false'>not this</p>\
                    <p>not this</p>"
                    .into(),
            },
        );
        assert_eq!(result, "<p>this</p><p>not this</p>");
    }

    #[test]
    fn pl_is() {
        let vars = json!({ "header": true });

        let result = render(
            &vars,
            &PathBuf::new(),
            &MockFilesystem {
                data: b"<p pl-is='header ? \"h1\" : \"h2\"'>this</p>".into(),
            },
        );
        assert_eq!(result, "<h1>this</h1>");
    }

    #[test]
    fn pl_html() {
        let vars = json!({ "content": "<p>hello world</p>" });

        let result = render(
            &vars,
            &PathBuf::new(),
            &MockFilesystem {
                data: b"<article pl-html='content'>something that used to be here</article>".into(),
            },
        );
        assert_eq!(result, "<article><p>hello world</p></article>");
    }
}
