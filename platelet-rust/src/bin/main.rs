use platelet::render_file;
use serde_json::Value;
use std::env;
use std::io::{self, Read};
use std::path::Path;

const USAGE: &str = "echo '{ \"some\": \"args\" }' | platelet [template.html]";

fn main() {
    let filename = env::args().nth(1).expect(USAGE);

    let mut stdin = String::new();
    io::stdin().read_to_string(&mut stdin).unwrap();
    let stdin: Value = serde_json::from_str(&stdin).unwrap();

    println!("{}", render_file(&stdin, &Path::new(&filename)).unwrap());
}
