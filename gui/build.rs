use std::process::{Command, exit};
use std::env;
use std::path::Path;

fn main() {
    let root_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let status = Command::new("elm")
        .args(&["make", "src/Main.elm", "--output", "static/index.html"])
        .current_dir(&Path::new(&root_dir))
        .status().unwrap();

    exit(status.code().expect("Elm compiler did not have an exit code"));
}
