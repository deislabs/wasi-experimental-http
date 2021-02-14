# `wasi-experimental-http`

This project aims to add temporary and experimental HTTP bindings for WASI runtimes.
It is currently unstable and incomplete.

### Building and running

```
$ cargo run --release
module instantiation time: 30.9532ms
wasi_experimental_http::data_from_memory:: length: 30
wasi_experimental_http::req: URL: https://api.brigade.sh/healthz
wasi_experimental_http: response: "OK"
wasi_experimental_http::write_guest_memory:: written 4 bytes
"OK"
```
