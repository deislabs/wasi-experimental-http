// @ts-ignore
import { Console } from "as-wasi";
import { Method, RequestBuilder, Response } from "../../crates/as";

export function post(): void {
  let body = String.UTF8.encode("testing the body");
  let res = new RequestBuilder("https://postman-echo.com/post")
    .header("Content-Type", "text/plain")
    .header("abc", "def")
    .method(Method.POST)
    .body(body)
    .send();

  check(res, 200, "content-type");
  res.close();
}

export function get(): void {
  let res = new RequestBuilder("https://some-random-api.ml/facts/dog")
    .method(Method.GET)
    .send();

  check(res, 200, "content-type");
  let bytes = res.bodyReadAll();
  let body = String.UTF8.decode(bytes.buffer);
  if (!body.includes("")) {
    Console.write("got " + body);
    abort();
  }
  res.close();
}

export function concurrent(): void {
  let req1 = makeReq();
  let req2 = makeReq();
  let req3 = makeReq();
}

function makeReq(): Response {
  return new RequestBuilder("https://some-random-api.ml/facts/dog")
    .method(Method.GET)
    .send();
}

function check(
  res: Response,
  expectedStatus: u32,
  expectedHeader: string
): void {
  if (res.status != expectedStatus) {
    Console.write(
      "expected status " +
        expectedStatus.toString() +
        " got " +
        res.status.toString()
    );
    abort();
  }

  let headerValue = res.headerGet(expectedHeader);
  if (!headerValue) {
    abort();
  }

  let headers = res.headerGetAll();
  if (headers.size == 0) {
    abort();
  }
}
