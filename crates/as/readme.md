# `@deislabs/wasi-experimental-http`

[![npm version](https://badge.fury.io/js/%40deislabs%2Fwasi-experimental-http.svg)](https://badge.fury.io/js/%40deislabs%2Fwasi-experimental-http)

Experimental HTTP client library for AssemblyScript.

### Using this library

First, install the package to your project:

```bash
$ npm install @deislabs/wasi-experimental-http --save
```

Then, import the package and create a request using the `RequestBuilder`:

```typescript
// @ts-ignore
import { Console } from "as-wasi";
import {
  Method,
  RequestBuilder,
  Response,
} from "@deislabs/wasi-experimental-http";

export function post(): void {
  let body = String.UTF8.encode("testing the body");
  let res = new RequestBuilder("https://postman-echo.com/post")
    .header("Content-Type", "text/plain")
    .method(Method.POST)
    .body(body)
    .send();

  print(res);
}

function print(res: Response): void {
  Console.log(res.status.toString());
  Console.log(res.getHeader("Content-Type"));
  let result = String.UTF8.decode(res.bodyReadAll().buffer);
  Console.log(result);
}
```

After building a WebAssembly module using the AssemblyScript compiler, the
module can be executed in a Wasmtime runtime that has the experimental HTTP
functionality enabled (the crate to configure it can be found in this repo):

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
