# `wasi-experimental-http`

This is an experiment intended to provide a _temporary_ workaround until the
WASI networking API is stable, and is compatible with [Wasmtime v0.26][24] by
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
    let str = std::str::from_utf8(&res.body_read_all()).unwrap().to_string();
    println!("{:#?}", res.header_get("Content-Type"));
    println!("{}", str);
    println!("{:#?}", res.status_code);
}
```

Build the module using the `wasm32-wasi` target, then follow the next section to
update a Wasmtime runtime with the experimental HTTP support.

### Adding support to a Wasmtime runtime

The easiest way to add support is by using the
[Wasmtime linker](https://docs.rs/wasmtime/0.26.0/wasmtime/struct.Linker.html):

```rust
let store = Store::default();
let mut linker = Linker::new(&store);
let wasi = Wasi::new(&store, ctx);

// link the WASI core functions
wasi.add_to_linker(&mut linker)?;

// link the experimental HTTP support
let allowed_hosts = Some(vec!["https://postman-echo.com".to_string()]);
let max_concurrent_requests = Some(42);

let http = HttpCtx::new(allowed_domains, max_concurrent_requests)?;
http.add_to_linker(&mut linker)?;
```

Then, executing the module above will send the HTTP request and write the
response:

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

The Wasmtime implementation also enables allowed hosts - an optional and
configurable list of domains or hosts that guest modules are allowed to send
requests to. If `None` or an empty vector is passed, guest modules are **NOT**
allowed to make HTTP requests to any server. (Note that the hosts passed MUST
have the protocol also specified - i.e. `https://my-domain.com`, or
`http://192.168.0.1`, and if making requests to a subdomain, the subdomain MUST
be in the allowed list. See the the library tests for more examples).

Note that the Wasmtime version currently supported is
[0.26](https://docs.rs/wasmtime/0.26.0/wasmtime/).

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

export function _start_(): void {
  let body = String.UTF8.encode("testing the body");
  let res = new RequestBuilder("https://postman-echo.com/post")
    .header("Content-Type", "text/plain")
    .method(Method.POST)
    .body(body)
    .send();
  wasi.Console.log(res.status.toString());
  wasi.Console.log(res.headersGetAll.toString());
  wasi.Console.log(String.UTF8.decode(res.bodyReadAll().buffer));
}
```

### Testing using the `wasmtime-http` binary

This project also adds a convenience binary for testing modules with HTTP
support, `wasmtime-http` - a simple program that mimics the `wasmtime run`
command, but also adds support for sending HTTP requests.

````
➜ cargo run --bin wasmtime-http -- --help
wasmtime-http 0.1.0

USAGE:
    wasmtime-http [OPTIONS] <module> [--] [ARGS]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --allowed-host <allowed-hosts>...    Host the guest module is allowed to make outbound HTTP requests to
    -i, --invoke <invoke>                    The name of the function to run [default: _start]
    -c, --concurrency <max-concurrency>      The maximum number of concurrent requests a module can make to allowed
                                             hosts
    -e, --env <NAME=VAL>...                  Pass an environment variable to the program

ARGS:
    <module>     The path of the WebAssembly module to run
    <ARGS>...    The arguments to pass to the module```
````

### Known limitations

- there is no support for streaming HTTP responses, which this means guest
  modules have to wait until the entire body has been written by the runtime
  before reading it.
- request and response bodies are [`Bytes`](https://docs.rs/bytes/1.0.1/bytes/).
- the current WITX definitions are experimental, and currently only used to
  generate guest bindings.
- this library does not aim to add support for running HTTP servers in
  WebAssembly.

### Code of Conduct

This project has adopted the
[Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct/).

For more information see the
[Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/) or
contact [opencode@microsoft.com](mailto:opencode@microsoft.com) with any
additional questions or comments.

[24]: https://github.com/bytecodealliance/wasmtime/releases/tag/v0.26.0
[sockets-wip]: https://github.com/WebAssembly/WASI/pull/312

```

```
