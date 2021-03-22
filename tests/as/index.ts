// @ts-ignore
import * as wasi from "as-wasi";
import { Method, RequestBuilder, Response } from "../../crates/as";
export { alloc } from "../../crates/as";

export function post(): void {
  let body = String.UTF8.encode("testing the body");
  let res = new RequestBuilder("https://postman-echo.com/post")
    .header("Content-Type", "text/plain")
    .header("abc", "def")
    .method(Method.POST)
    .body(body)
    .send();

  checkStatus(res.status);
}

export function get(): void {
  let res = new RequestBuilder("https://api.brigade.sh/healthz")
    .method(Method.GET)
    .send();

  checkStatus(res.status);
  if (String.UTF8.decode(res.body) != '"OK"') {
    abort();
    abort();
  }
}

function checkStatus(status: number): void {
  if (status != 200) {
    abort();
  }
}
