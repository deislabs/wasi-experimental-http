# `wasi-experimental-http`

This project aims to add temporary and experimental HTTP bindings for WASI runtimes.
It is currently unstable and incomplete.

### Using this library

We use the `wasi-http` crate (from this repository) and the `http` crate and make an HTTP request from the WebAssembly module:

```rust
use http;
use wasi_http;

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let url = "https://api.brigade.sh/healthz".to_string();
    let req = http::request::Builder::new().uri(&url).body(()).unwrap();
    let res = wasi_http::request(req).unwrap();
    let str = std::str::from_utf8(&res.body()).unwrap().to_string();
    println!("{}", str);
}
```

Then, we use the runtime, with the additional `wasi_experimental_http` module linked, and execute the module from above:

```
$ cargo run --release
module instantiation time: 30.9532ms
wasi_experimental_http::data_from_memory:: length: 30
wasi_experimental_http::req: URL: https://api.brigade.sh/healthz
wasi_experimental_http: response: "OK"
wasi_experimental_http::write_guest_memory:: written 4 bytes
"OK"
```
