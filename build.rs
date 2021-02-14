use std::process;

const TESTS_DIR: &str = "./tests";
const SIMPLE_EXAMPLE: &str = "simple";

fn main() {
    cargo_build_example(TESTS_DIR.to_string(), SIMPLE_EXAMPLE.to_string())
}

fn cargo_build_example(dir: String, example: String) {
    let input = format!("{}/{}.rs", dir, example);
    let output = format!("{}/{}.wasm", dir, example);

    // println!("cargo:rerun-if-changed={}", input);

    let mut cmd = process::Command::new("rustc");

    cmd.stdout(process::Stdio::piped());
    cmd.stderr(process::Stdio::piped());
    cmd.arg("--target")
        .arg("wasm32-wasi")
        .arg(input)
        .arg("-o")
        .arg(output)
        .arg("--crate-type=cdylib");
    cmd.output().unwrap();
}
