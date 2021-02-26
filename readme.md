# `wasi-experimental-http`

This projects adds experimental and _temporary_ outgoing HTTP support for WASI
modules. It is currently unstable and incomplete.

### Using this library

We use the `wasi-http` crate (from this repository) and the `http` crate and
make an HTTP request from the WebAssembly module:

### Writing a module that makes an HTTP request

We use the `wasi-experimental-http` crate (from this repository) and the `http`
crate to create an HTTP request from a WebAssembly module, make a host call to
the runtime using the request, then get a response back:

```rust
use http;
use wasi_experimental_http;

#[no_mangle]
pub extern "C" fn _start() {
    let url = "https://postman-echo.com/post".to_string();
    let req = http::request::Builder::new()
        .method(http::Method::POST)
        .uri(&url)
        .header("Content-Type", "text/plain")
        .header("abc", "def");
    let b = b"Testing with a request body. Does this actually work?";
    let req = req.body(Some(b.to_vec())).unwrap();

    let res = wasi_experimental_http::request(req).expect("cannot make request");
    let str = std::str::from_utf8(&res.body()).unwrap().to_string();
    println!("{:#?}", res.headers());
    println!("{}", str);
    println!("{:#?}", res.status().to_string());
}
```

Build the module using the `wasm32-wasi` target, then assuming the runtime
supports this library (currently it has to be built on top of Wasmtime), the
module can be instantiated, and the runtime can send the request and write the
response back to the guest module:

```
$ cargo run
wasi_experimental_http::data_from_memory:: length: 29
wasi_experimental_http::data_from_memory:: length: 41
wasi_experimental_http::data_from_memory:: length: 4
wasi_experimental_http::data_from_memory:: length: 53
wasi_experimental_http::write_guest_memory:: written 336 bytes
wasi_experimental_http::write_guest_memory:: written 374 bytes
{
    "content-length": "374",
    "connection": "keep-alive",
    "set-cookie": "sails.sid=s%3AqFVoX_OuCEaG0vtZ2sjldxVLbphpRYNi.KxWbUPNDQ%2BuMaAu9IICiEBDxsAd0RJ8uvUi9YMN3Nv8; Path=/; HttpOnly",
    "vary": "Accept-Encoding",
    "content-type": "application/json; charset=utf-8",
    "date": "Fri, 26 Feb 2021 18:31:03 GMT",
    "etag": "W/\"176-Ky4OTmr3Xbcl3yNah8w2XIQapGU\"",
}
{"args":{},"data":"Testing with a request body. Does this actually work?","files":{},"form":{},"headers":{"x-forwarded-proto":"https","x-forwarded-port":"443","host":"postman-echo.com","x-amzn-trace-id":"Root=1-60393e67-02d1c8033bcf4f1e74a4523e","content-length":"53","content-type":"text/plain","abc":"def","accept":"*/*"},"json":null,"url":"https://postman-echo.com/post"}
"200 OK"
```

### Known limitations

- request and response bodies are byte arrays only (Rust `Vec<u8>`).
- handling headers is currently very inefficient.
- there are no WITX definitions, which means we have to manually keep the host
  call and guest implementation in sync. Adding WITX definitions could also
  enable support for other WASI runtimes.
- the AssemblyScript implementation is currently incomplete.
