# `wasi-experimental-http`

![Crates.io](https://img.shields.io/crates/v/wasi-experimental-http)

Experimental HTTP library for WebAssembly

### Using the crate

First, add this crate to your project. Then, it can be used to create and send
an HTTP request to a server:

```rust
use bytes::Bytes;
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
    let b = Bytes::from("Testing with a request body. Does this actually work?");
    let req = req.body(Some(b)).unwrap();

    let res = wasi_experimental_http::request(req).expect("cannot make request");
    let str = std::str::from_utf8(&res.body_read_all()).unwrap().to_string();
    println!("{:#?}", res.header_get("Content-Type"));
    println!("{}", str);
    println!("{:#?}", res.status_code);
}
```

Build the module using the `wasm32-wasi` target, then execute in a Wasmtime
runtime that has the experimental HTTP functionality enabled (the crate to
configure it can be found in this repo):

```
{
    "content-length": "374",
    "connection": "keep-alive",
    "set-cookie": "sails.Path=/; HttpOnly",
    "vary": "Accept-Encoding",
    "content-type": "application/json; charset=utf-8",
    "date": "Fri, 26 Feb 2021 18:31:03 GMT",
    "etag": "W/\"176-Ky4OTmr3Xbcl3yNah8w2XIQapGU\"",
}
{"args":{},"data":"Testing with a request body. Does this actually work?","files":{},"form":{},"headers":{"x-forwarded-proto":"https","x-forwarded-port":"443","host":"postman-echo.com","x-amzn-trace-id":"Root=1-60393e67-02d1c8033bcf4f1e74a4523e","content-length":"53","content-type":"text/plain","abc":"def","accept":"*/*"},"json":null,"url":"https://postman-echo.com/post"}
"200 OK"
```
