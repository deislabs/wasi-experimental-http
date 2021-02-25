use std::process;

const TESTS_DIR: &str = "tests";
const SIMPLE_EXAMPLE: &str = "simple";

fn main() {
    println!("cargo:rerun-if-changed=tests/simple/src/lib.rs");
    println!("cargo:rerun-if-changed=crates/wasi-experimental-http/src/lib.rs");

    cargo_build_example(TESTS_DIR.to_string(), SIMPLE_EXAMPLE.to_string())
}

fn cargo_build_example(dir: String, example: String) {
    let dir = format!("{}/{}", dir, example);

    let mut cmd = process::Command::new("cargo");
    cmd.current_dir(dir);
    cmd.stdout(process::Stdio::piped());
    cmd.stderr(process::Stdio::piped());
    cmd.arg("build")
        .arg("--target")
        .arg("wasm32-wasi")
        .arg("--release");
    cmd.output().unwrap();
}
