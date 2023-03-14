use std::{
    ffi::OsStr,
    path::{Component, PathBuf},
};

use anyhow::{bail, Error};
use structopt::StructOpt;
use wasi_cap_std_sync::WasiCtxBuilder;
use wasi_experimental_http_wasmtime::{HttpCtx, HttpState};
use wasmtime::{AsContextMut, Engine, Func, Instance, Linker, Store, Val, ValType};
use wasmtime_wasi::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "wasmtime-http")]
struct Opt {
    #[structopt(help = "The path of the WebAssembly module to run")]
    module: String,

    #[structopt(
        short = "i",
        long = "invoke",
        default_value = "_start",
        help = "The name of the function to run"
    )]
    invoke: String,

    #[structopt(
        short = "e",
        long = "env",
        value_name = "NAME=VAL",
        parse(try_from_str = parse_env_var),
        help = "Pass an environment variable to the program"
    )]
    vars: Vec<(String, String)>,

    #[structopt(
        short = "a",
        long = "allowed-host",
        help = "Host the guest module is allowed to make outbound HTTP requests to"
    )]
    allowed_hosts: Option<Vec<String>>,

    #[structopt(
        short = "c",
        long = "concurrency",
        help = "The maximum number of concurrent requests a module can make to allowed hosts"
    )]
    max_concurrency: Option<u32>,

    #[structopt(value_name = "ARGS", help = "The arguments to pass to the module")]
    module_args: Vec<String>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    let method = opt.invoke.clone();
    // println!("{:?}", opt);
    let (instance, mut store) = create_instance(
        opt.module,
        opt.vars,
        opt.module_args.clone(),
        opt.allowed_hosts,
        opt.max_concurrency,
    )?;
    let func = instance
        .get_func(&mut store, method.as_str())
        .unwrap_or_else(|| panic!("cannot find function {}", method));

    invoke_func(func, opt.module_args, &mut store)?;

    Ok(())
}

fn create_instance(
    filename: String,
    vars: Vec<(String, String)>,
    args: Vec<String>,
    allowed_hosts: Option<Vec<String>>,
    max_concurrent_requests: Option<u32>,
) -> Result<(Instance, Store<WasmtimeHttpCtx>), Error> {
    let mut wasmtime_config = wasmtime::Config::default();
    wasmtime_config.wasm_multi_memory(true);
    let engine = Engine::new(&wasmtime_config)?;
    let mut linker = Linker::new(&engine);

    let args = compute_argv(filename.clone(), &args);

    let wasi = WasiCtxBuilder::new()
        .inherit_stdin()
        .inherit_stdout()
        .inherit_stderr()
        .envs(&vars)?
        .args(&args)?
        .build();

    let http = HttpCtx {
        allowed_hosts,
        max_concurrent_requests,
    };

    let ctx = WasmtimeHttpCtx { wasi, http };

    let mut store = Store::new(&engine, ctx);
    wasmtime_wasi::add_to_linker(&mut linker, |cx: &mut WasmtimeHttpCtx| -> &mut WasiCtx {
        &mut cx.wasi
    })?;
    // Link `wasi_experimental_http`
    let http = HttpState::new()?;
    http.add_to_linker(&mut linker, |cx: &WasmtimeHttpCtx| -> HttpCtx {
        cx.http.clone()
    })?;

    let module = wasmtime::Module::from_file(store.engine(), filename)?;
    let instance = linker.instantiate(&mut store, &module)?;

    Ok((instance, store))
}

// Invoke function given module arguments and print results.
// Adapted from https://github.com/bytecodealliance/wasmtime/blob/main/src/commands/run.rs.
fn invoke_func(func: Func, args: Vec<String>, mut store: impl AsContextMut) -> Result<(), Error> {
    let ty = func.ty(&mut store);

    let mut args = args.iter();
    let mut values = Vec::new();
    for ty in ty.params() {
        let val = match args.next() {
            Some(s) => s,
            None => {
                bail!("not enough arguments for invocation")
            }
        };
        values.push(match ty {
            ValType::I32 => Val::I32(val.parse()?),
            ValType::I64 => Val::I64(val.parse()?),
            ValType::F32 => Val::F32(val.parse()?),
            ValType::F64 => Val::F64(val.parse()?),
            t => bail!("unsupported argument type {:?}", t),
        });
    }

    let mut results = vec![];
    func.call(&mut store, &values, &mut results)?;
    for result in results {
        match result {
            Val::I32(i) => println!("{}", i),
            Val::I64(i) => println!("{}", i),
            Val::F32(f) => println!("{}", f32::from_bits(f)),
            Val::F64(f) => println!("{}", f64::from_bits(f)),
            Val::ExternRef(_) => println!("<externref>"),
            Val::FuncRef(_) => println!("<funcref>"),
            Val::V128(i) => println!("{}", i),
        };
    }

    Ok(())
}

fn parse_env_var(s: &str) -> Result<(String, String), Error> {
    let parts: Vec<_> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        bail!("must be of the form `key=value`");
    }
    Ok((parts[0].to_owned(), parts[1].to_owned()))
}

fn compute_argv(module: String, args: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    let module = PathBuf::from(module);
    // Add argv[0], which is the program name. Only include the base name of the
    // main wasm module, to avoid leaking path information.
    result.push(
        module
            .components()
            .next_back()
            .map(Component::as_os_str)
            .and_then(OsStr::to_str)
            .unwrap_or("")
            .to_owned(),
    );

    // Add the remaining arguments.
    for arg in args.iter() {
        result.push(arg.clone());
    }

    result
}

struct WasmtimeHttpCtx {
    pub wasi: WasiCtx,
    pub http: HttpCtx,
}
