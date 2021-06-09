use std::process::{self, Command};

const TESTS_DIR: &str = "tests";
const RUST_EXAMPLE: &str = "rust";
const AS_EXAMPLE: &str = "as";

const RUST_GUEST_RAW: &str = "crates/wasi-experimental-http/src/raw.rs";
const AS_GUEST_RAW: &str = "crates/as/raw.ts";
const MD_GUEST_API: &str = "witx/readme.md";

const WITX_CODEGEN_VERSION: &str = "0.11.0";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=tests/rust/src/lib.rs");
    println!("cargo:rerun-if-changed=crates/wasi-experimental-http/src/lib.rs");
    println!("cargo:rerun-if-changed=tests/as/index.ts");
    println!("cargo:rerun-if-changed=crates/as/index.ts");
    println!("cargo:rerun-if-changed=witx/wasi_experimental_http.witx");

    generate_from_witx("rust", RUST_GUEST_RAW);
    generate_from_witx("assemblyscript", AS_GUEST_RAW);
    generate_from_witx("markdown", MD_GUEST_API);

    cargo_build_example(TESTS_DIR, RUST_EXAMPLE);
    as_build_example(TESTS_DIR, AS_EXAMPLE);
}

fn cargo_build_example(dir: &str, example: &str) {
    let dir = format!("{}/{}", dir, example);

    run(
        vec!["cargo", "build", "--target", "wasm32-wasi", "--release"],
        Some(dir),
    );
}

fn as_build_example(dir: &str, example: &str) {
    let dir = format!("{}/{}", dir, example);

    run(vec!["npm", "install"], Some(dir.clone()));
    run(vec!["npm", "run", "asbuild"], Some(dir));
}

fn check_witx_codegen() {
    match process::Command::new("witx-codegen").spawn() {
        Ok(_) => {
            eprintln!("witx-codegen already installed");
        }
        Err(_) => {
            println!("cannot find witx-codegen, attempting to install");
            run(
                vec![
                    "cargo",
                    "install",
                    "witx-codegen",
                    "--version",
                    WITX_CODEGEN_VERSION,
                ],
                None,
            );
        }
    }
}

fn generate_from_witx(codegen_type: &str, output: &str) {
    check_witx_codegen();

    run(
        vec![
            "witx-codegen",
            "--output-type",
            codegen_type,
            "--output",
            output,
            "witx/wasi_experimental_http.witx",
        ],
        None,
    );
}

fn run<S: Into<String> + AsRef<std::ffi::OsStr>>(args: Vec<S>, dir: Option<String>) {
    let mut cmd = Command::new(get_os_process());
    cmd.stdout(process::Stdio::piped());
    cmd.stderr(process::Stdio::piped());

    if let Some(dir) = dir {
        cmd.current_dir(dir);
    };

    cmd.arg("-c");
    cmd.arg(
        args.into_iter()
            .map(Into::into)
            .collect::<Vec<String>>()
            .join(" "),
    );

    println!("running {:#?}", cmd);

    cmd.output().unwrap();
}

fn get_os_process() -> String {
    if cfg!(target_os = "windows") {
        String::from("powershell.exe")
    } else {
        String::from("/bin/bash")
    }
}
