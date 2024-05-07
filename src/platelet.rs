use lol_html::html_content::ContentType;
use lol_html::{element, text, HtmlRewriter, Settings};
use serde_json::Value;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};

const USAGE: &str = "echo '{ \"some\": \"args\" }' | platelet [template.html]";

pub enum SrcAndData {
    BothSrcAndData(String, Value),
    JustSrc(String),
    JustData(Value),
}

enum Replacement {
    Template(SrcAndData),
}

fn main() {
    let filename = env::args().nth(1).expect(USAGE);
    let filename = Path::new(&filename);

    let mut stdin = String::new();
    io::stdin().read_to_string(&mut stdin).unwrap();
    let stdin: Value = serde_json::from_str(&stdin).unwrap();

    println!("{}", run(stdin, filename.to_path_buf()));
}

fn run(props: Value, filename: PathBuf) -> String {
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
                            let rendered = run(data, path.clone());
                            el.replace(&rendered, ContentType::Html)
                        }
                        None => {}
                    }
                    Ok(())
                }),
                text!("*", |txt| {
                    if txt.as_str().starts_with("{") {
                        let key = txt.as_str().to_string();
                        let key = key.strip_prefix("{").unwrap().strip_suffix("}").unwrap();
                        let key: Vec<&str> = key.split(".").collect();
                        let val = get_in(&key, &props);
                        txt.replace(val.as_str().unwrap(), ContentType::Text)
                    }
                    Ok(())
                }),
            ],
            ..Settings::default()
        },
        |c: &[u8]| out.extend_from_slice(c),
    );
    let file = File::open(filename.clone()).expect("bad file");
    let mut file = BufReader::new(file);

    loop {
        let buf = file.fill_buf().unwrap();
        let len = buf.len();
        if len > 0 {
            rewriter.write(buf).expect("can't write to rewriter");
            file.consume(len);
        } else {
            break;
        }
    }

    rewriter.end().expect("no end");

    String::from_utf8(out).unwrap()
}

fn get_in<'a>(key: &[&str], val: &'a Value) -> &'a Value {
    match key {
        [x, ..] => match val.as_object().unwrap().get(*x) {
            Some(zzzz) => match key {
                [_] => zzzz,
                _ => get_in(&key[1..], zzzz),
            },
            None => panic!("{x}, {:?}", val),
        },
        _ => panic!("[]"),
    }
}
