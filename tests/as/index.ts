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

  check(res, 200, 7);
}

export function get(): void {
  let res = new RequestBuilder("https://api.brigade.sh/healthz")
    .method(Method.GET)
    .send();

  check(res, 200, 6);
  if (String.UTF8.decode(res.body) != '"OK"') {
    abort();
  }
}

function check(
  res: Response,
  expectedStatus: u32,
  expectedHeadersLen: u32
): void {
  if (res.status != expectedStatus) {
    wasi.Console.write(
      "expected status " +
        expectedStatus.toString() +
        " got " +
        res.status.toString()
    );
    abort();
  }

  let len = (res.headers.keys() as Array<string>).length;
  if (len != expectedHeadersLen) {
    wasi.Console.write(
      "expected " +
        expectedHeadersLen.toString() +
        " headers, got " +
        len.toString()
    );
    abort();
  }
}
