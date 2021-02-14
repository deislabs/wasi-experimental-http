use anyhow::Error;
use std::time::Instant;
use wasi_experimental_http::link_http;
use wasmtime::*;
use wasmtime_wasi::{Wasi, WasiCtxBuilder};

const START_FN: &str = "_start";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let instance = create_instance("tests/simple.wasm".to_string())?;
    run_start(&instance)
}

/// Execute the module's `_start` function.
fn run_start(instance: &Instance) -> Result<(), Error> {
    let entrypoint = instance
        .get_func(START_FN)
        .expect("expected alloc function not found");
    entrypoint.call(&vec![])?;

    Ok(())
}

/// Create a Wasmtime::Instance from a compiled module and
/// link the WASI imports.
fn create_instance(filename: String) -> Result<Instance, Error> {
    let start = Instant::now();
    let store = Store::default();
    let mut linker = Linker::new(&store);

    let ctx = WasiCtxBuilder::new()
        .inherit_stdin()
        .inherit_stdout()
        .inherit_stderr()
        .build()?;

    let wasi = Wasi::new(&store, ctx);
    wasi.add_to_linker(&mut linker)?;
    // Link `wasi_experimental_http::req`.
    link_http(&mut linker)?;

    let module = wasmtime::Module::from_file(store.engine(), filename)?;

    let instance = linker.instantiate(&module)?;
    let duration = start.elapsed();
    println!("module instantiation time: {:#?}", duration);
    Ok(instance)
}
