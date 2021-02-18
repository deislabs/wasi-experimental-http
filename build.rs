use std::process;

const TESTS_DIR: &str = "./tests";
const SIMPLE_EXAMPLE: &str = "simple";

fn main() {
    cargo_build_example(TESTS_DIR.to_string(), SIMPLE_EXAMPLE.to_string())
}

fn cargo_build_example(dir: String, example: String) {
    let proj = format!("{}/{}/Cargo.toml", dir, example);

    let mut cmd = process::Command::new("cargo");
    cmd.stdout(process::Stdio::piped());
    cmd.stderr(process::Stdio::piped());
    cmd.arg("build")
        .arg("--target")
        .arg("wasm32-wasi")
        .arg("--manifest-path")
        .arg(proj);
    cmd.output().unwrap();
}
