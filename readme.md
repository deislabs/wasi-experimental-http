# `wasi-experimental-http`

This is an experiment intended to provide a _temporary_ workaround until the
WASI networking API is stable, and is compatible with [Wasmtime v0.24][24] by
using the `wasi_experiemental_http_wasmtime` crate. We expect that once [the
WASI sockets proposal][sockets-wip] gets adopted and implemented in language
toolchains, the need for this library will vanish.

### Writing a module that makes an HTTP request

We use the `wasi-experimental-http` crate (from this repository) and the `http`
crate to create an HTTP request from a WebAssembly module, make a host call to
the runtime using the request, then get a response back:

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
    let str = std::str::from_utf8(&res.body()).unwrap().to_string();
    println!("{:#?}", res.headers());
    println!("{}", str);
    println!("{:#?}", res.status().to_string());
}
```

Build the module using the `wasm32-wasi` target, then follow the next section to
update a Wasmtime runtime with the experimental HTTP support.

### Adding support to a Wasmtime runtime

The easiest way to add support is by using the
[Wasmtime linker](https://docs.rs/wasmtime/0.24.0/wasmtime/struct.Linker.html):

```rust
let store = Store::default();
let mut linker = Linker::new(&store);
let wasi = Wasi::new(&store, ctx);

// link the WASI core functions
wasi.add_to_linker(&mut linker)?;

// link the experimental HTTP support
wasi_experimental_http_wasmtime::link_http(&mut linker, None)?;
```

Then, executing the module above will send the HTTP request and write the
response:

```
wasi_experimental_http::data_from_memory:: length: 29
wasi_experimental_http::data_from_memory:: length: 41
wasi_experimental_http::data_from_memory:: length: 4
wasi_experimental_http::data_from_memory:: length: 53
wasi_experimental_http::write_guest_memory:: written 336 bytes
wasi_experimental_http::write_guest_memory:: written 374 bytes
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

The Wasmtime implementation also enables allowed domains - an optional and
configurable list of domains or hosts that guest modules are allowed to send
requests to. If `None` is passed, guest modules are allowed to access any domain
or host. (Note that the hosts passed MUST have the protocol also specified -
i.e. `https://my-domain.com`, or `http://192.168.0.1`, and if making requests to
a subdomain, the subdomain MUST be in the allowed list. See the the library
tests for more examples).

Note that the Wasmtime version currently supported is
[0.24](https://docs.rs/wasmtime/0.24.0/wasmtime/).

### Sending HTTP requests from AssemblyScript

This repository also contains an AssemblyScript implementation for sending HTTP
requests:

```typescript
// @ts-ignore
import * as wasi from "as-wasi";
import {
  Method,
  RequestBuilder,
  Response,
} from "@deislabs/wasi-experimental-http";
export { alloc } from "@deislabs/wasi-experimental-http";

export function _start_(): void {
  let body = String.UTF8.encode("testing the body");
  let res = new RequestBuilder("https://postman-echo.com/post")
    .header("Content-Type", "text/plain")
    .method(Method.POST)
    .body(body)
    .send();
  wasi.Console.log(res.status.toString());
  wasi.Console.log(res.headers);
  wasi.Console.log(String.UTF8.decode(res.body));
}
```

### Known limitations

- there is no support for streaming HTTP responses, which this means guest
  modules have to wait until the entire body has been written by the runtime
  before reading it.
- request and response bodies are [`Bytes`](https://docs.rs/bytes/1.0.1/bytes/).
- handling headers in Rust is currently very inefficient.
- there are no WITX definitions, which means we have to manually keep the host
  call and guest implementation in sync. Adding WITX definitions could also
  enable support for other WASI runtimes.
- the `alloc` function in AssemblyScript has to be re-exported in order for the
  runtime to use it. There probably is a decorator or Binaryen setting to avoid
  removing it, but for now this has to be re-exported.
- this library does not aim to add support for running HTTP servers in
  WebAssembly.

### Code of Conduct

This project has adopted the
[Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct/).

For more information see the
[Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/) or
contact [opencode@microsoft.com](mailto:opencode@microsoft.com) with any
additional questions or comments.

[24]: https://github.com/bytecodealliance/wasmtime/releases/tag/v0.24.0
[sockets-wip]: https://github.com/WebAssembly/WASI/pull/312
