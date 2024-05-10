use platelet::renderer::{render_to_string, Filesystem};
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
    fn get_data_at_path(&self, filename: &PathBuf) -> String {
        let mut file = File::open(filename.clone()).expect("bad file");
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        buf
    }
}

fn main() {
    let filename = env::args().nth(1).expect(USAGE);
    let filename = Path::new(&filename);

    let mut stdin = String::new();
    io::stdin().read_to_string(&mut stdin).unwrap();
    let stdin: Value = serde_json::from_str(&stdin).unwrap();

    println!(
        "{}",
        render_to_string(
            &stdin,
            &filename.to_path_buf(),
            &RealFilesystem::RealFilesystem
        )
    );
}
