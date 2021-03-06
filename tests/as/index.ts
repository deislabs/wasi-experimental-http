// @ts-ignore
import * as wasi from "as-wasi";
import { Method, RequestBuilder, Response } from "../../crates/as";
export { alloc } from "../../crates/as";

export function post(): void {
  let body = String.UTF8.encode("testing the body");
  let res = new RequestBuilder("https://postman-echo.com/post")
    .header("Content-Type", "text/plain")
    .method(Method.POST)
    .body(body)
    .send();

  let result = String.UTF8.decode(res.body);
  // print(res);
}

export function get(): void {
  let res = new RequestBuilder("https://api.brigade.sh/healthz")
    .method(Method.GET)
    .send();
  // print(res);
  if (String.UTF8.decode(res.body) != '"OK"') {
    abort();
  }
}

function print(res: Response): void {
  wasi.Console.log(res.status.toString());
  wasi.Console.log(res.headers);
  let result = String.UTF8.decode(res.body);
  wasi.Console.log(result);
}
