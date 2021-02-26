use anyhow::Error;
use std::time::Instant;
use wasi_cap_std_sync::WasiCtxBuilder;
use wasi_experimental_http_wasmtime::link_http;
use wasmtime::*;
use wasmtime_wasi::Wasi;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let instance =
        create_instance("target/wasm32-wasi/release/simple_wasi_http_tests.wasm".to_string())?;
    // let instance = create_instance("crates/as/build/optimized.wasm".to_string())?;

    let test_funcs = vec!["get", "post"];
    run_start(&instance, &test_funcs)
}

/// Execute the module's `_start` function.
fn run_start(instance: &Instance, test_funcs: &Vec<&str>) -> Result<(), Error> {
    for func in test_funcs.iter() {
        let func = instance.get_func(func).expect("cannot find function");
        func.call(&vec![])?;
    }

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
