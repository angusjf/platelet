use platelet::render_to_string;
use platelet::renderer::Filesystem;
use serde_json::Value;
use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

const USAGE: &str = "echo '{ \"some\": \"args\" }' | platelet [template.html]";

enum RealFilesystem {
    RealFilesystem,
}

impl Filesystem for RealFilesystem {
    fn read(&self, filename: &String) -> String {
        let filename: PathBuf = filename.try_into().unwrap();
        let mut file = File::open(filename.clone()).expect("bad file");
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        buf
    }
    fn move_to(&self, current: &String, path: &String) -> String {
        let current: PathBuf = current.try_into().unwrap();
        current
            .parent()
            .unwrap()
            .join(path)
            .to_str()
            .unwrap()
            .to_owned()
    }
}

fn main() {
    let filename = env::args().nth(1).expect(USAGE);

    let mut stdin = String::new();
    io::stdin().read_to_string(&mut stdin).unwrap();
    let stdin: Value = serde_json::from_str(&stdin).unwrap();

    println!(
        "{}",
        render_to_string(&stdin, &filename, &RealFilesystem::RealFilesystem).unwrap()
    );
}
