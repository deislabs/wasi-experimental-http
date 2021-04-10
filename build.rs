use std::process;

const TESTS_DIR: &str = "tests";
const RUST_EXAMPLE: &str = "rust";
const AS_EXAMPLE: &str = "as";

const RUST_GUEST_RAW: &str = "crates/wasi-experimental-http/src/raw.rs";
const AS_GUEST_RAW: &str = "crates/as/raw.ts";
const MD_GUEST_API: &str = "witx/readme.md";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=tests/rust/src/lib.rs");
    println!("cargo:rerun-if-changed=crates/wasi-experimental-http/src/lib.rs");
    println!("cargo:rerun-if-changed=tests/as/index.ts");
    println!("cargo:rerun-if-changed=crates/as/index.ts");

    generate_from_witx("rust".to_string(), RUST_GUEST_RAW.to_string());
    generate_from_witx("assemblyscript".to_string(), AS_GUEST_RAW.to_string());
    generate_from_witx("markdown".to_string(), MD_GUEST_API.to_string());

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

fn check_witx_codegen() {
    match process::Command::new("witx-codegen").spawn() {
        Ok(_) => {
            eprintln!("witx-codegen already installed");
        }
        Err(e) => {
            if let std::io::ErrorKind::NotFound = e.kind() {
                let mut cmd = process::Command::new("cargo");
                cmd.stdout(process::Stdio::piped());
                cmd.stderr(process::Stdio::piped());
                cmd.arg("install").arg("witx-codegen");
                cmd.output().unwrap();
            } else {
                eprintln!("cannot find or install witx-codegen: {}", e);
            }
        }
    }
}

fn generate_from_witx(codegen_type: String, output: String) {
    check_witx_codegen();
    let mut cmd = process::Command::new("witx-codegen");
    cmd.stdout(process::Stdio::piped());
    cmd.stderr(process::Stdio::piped());
    cmd.arg("--output-type")
        .arg(codegen_type)
        .arg("--output")
        .arg(output)
        .arg("witx/wasi_experimental_http.witx");

    eprintln!("test {:#?}", cmd);
    cmd.output().unwrap();
}
