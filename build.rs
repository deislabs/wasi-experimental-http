use std::process;

const TESTS_DIR: &str = "tests";
const RUST_EXAMPLE: &str = "rust";
const AS_EXAMPLE: &str = "as";

fn main() {
    println!("cargo:rerun-if-changed=tests/rust/src/lib.rs");
    println!("cargo:rerun-if-changed=crates/wasi-experimental-http/src/lib.rs");
    println!("cargo:rerun-if-changed=tests/as/index.ts");
    println!("cargo:rerun-if-changed=crates/as/index.ts");

    cargo_build_example(TESTS_DIR.to_string(), RUST_EXAMPLE.to_string());
    as_build_example(TESTS_DIR.to_string(), AS_EXAMPLE.to_string());
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

fn as_build_example(dir: String, example: String) {
    let dir = format!("{}/{}", dir, example);

    let mut cmd = process::Command::new("npm");
    cmd.current_dir(dir.clone());
    cmd.stdout(process::Stdio::piped());
    cmd.stderr(process::Stdio::piped());
    cmd.arg("install");
    cmd.output().unwrap();

    let mut cmd = process::Command::new("npm");
    cmd.current_dir(dir);
    cmd.stdout(process::Stdio::piped());
    cmd.stderr(process::Stdio::piped());
    cmd.arg("run").arg("asbuild");
    cmd.output().unwrap();
}
