// required to use --abort=as-wasi
// @ts-ignore
import { Console } from "as-wasi";
import * as raw from "./raw";

/** Send an HTTP request and return an HTTP response.
 *
 * It is recommended to use the `RequestBuilder` helper class.
 */
export function request(req: Request): Response {
  return raw_request(
    req.url,
    methodEnumToString(req.method),
    headersToString(req.headers),
    req.body
  );
}

/** An HTTP response. */
export class Response {
  /** The HTTP response status code. */
  public status: StatusCode;

  /** The response handle */
  handle: raw.ResponseHandle;

  constructor(status: u16, handle: u32) {
    this.status = status;
    this.handle = handle;
  }

  /** Read a part of the response body in a supplied buffer */
  public bodyRead(buffer: ArrayBuffer): usize {
    let buf_read_ptr = memory.data(8);
    if (
      raw.bodyRead(
        this.handle,
        changetype<usize>(buffer),
        buffer.byteLength,
        buf_read_ptr
      ) != 0
    ) {
      return 0;
    }
    return load<isize>(buf_read_ptr);
  }

  /** Read the entire response body */
  public bodyReadAll(): Uint8Array {
    let chunk = new Uint8Array(4096);
    let buf = new Array<u8>();
    while (true) {
      let count = this.bodyRead(chunk.buffer);
      if (count <= 0) {
        return changetype<Uint8Array>(buf);
      }
      for (let i: u32 = 0; i < (count as u32); i++) {
        buf.push(chunk[i]);
      }
    }
  }

  /** Get a single header value given its key */
  public headerGet(name: string): string {
    let name_buf = String.UTF8.encode(name);
    let name_ptr = changetype<usize>(name_buf);
    let name_len = name_buf.byteLength;

    let value_buf = new Uint8Array(4096);
    let value_buf_ptr = changetype<usize>(value_buf.buffer);
    let value_buf_len = value_buf.byteLength;
    let value_len_ptr = memory.data(8);

    if (
      raw.headerGet(
        this.handle,
        name_ptr,
        name_len,
        value_buf_ptr,
        value_buf_len,
        value_len_ptr
      ) != 0
    ) {
      return "";
    }

    let value = value_buf.subarray(0, load<u32>(value_len_ptr));
    return String.UTF8.decode(value.buffer);
  }

  /** Read all response headers into a header map */
  public headerGetAll(): Map<string, string> {
    let headers_buf = new Uint8Array(4 * 1024);
    let headers_buf_ptr = changetype<usize>(headers_buf.buffer);
    let headers_len_ptr = memory.data(8);

    if (
      raw.headersGetAll(
        this.handle,
        headers_buf_ptr,
        headers_buf.byteLength,
        headers_len_ptr
      ) != 0
    ) {
      return new Map<string, string>();
    }

    let headers = String.UTF8.decode(
      headers_buf.subarray(0, load<u32>(headers_len_ptr)).buffer
    );
    return stringToHeaderMap(headers);
  }

  public close(): void {
    raw.close(this.handle);
  }
}

/** An HTTP request.
 *
 * It is recommended to use the a `RequestBuilder`
 * to create and send HTTP requests.
 */
export class Request {
  /** The URL of the request. */
  public url: string;
  /** The HTTP method of the request. */
  public method: Method;
  /** The request headers. */
  public headers: Map<string, string>;
  /** The request body as bytes. */
  public body: ArrayBuffer;

  constructor(
    url: string,
    method: Method = Method.GET,
    headers: Map<string, string> = new Map<string, string>(),
    body: ArrayBuffer = new ArrayBuffer(0)
  ) {
    this.url = url;
    this.method = method;
    this.headers = headers;
    this.body = body;
  }
}

export class RequestBuilder {
  private request: Request;

  constructor(url: string) {
    this.request = new Request(url);
  }

  /** Set the request's HTTP method. */
  public method(m: Method): RequestBuilder {
    this.request.method = m;
    return this;
  }

  /** Add a new pair of header key and header value to the request. */
  public header(key: string, value: string): RequestBuilder {
    this.request.headers.set(key, value);
    return this;
  }

  /** Set the request's body. */
  public body(b: ArrayBuffer): RequestBuilder {
    this.request.body = b;
    return this;
  }

  /** Send the request and return an HTTP response. */
  public send(): Response {
    return request(this.request);
  }
}

function raw_request(
  url: string,
  method: string,
  headers: string,
  body: ArrayBuffer
): Response {
  let url_buf = String.UTF8.encode(url);
  let url_ptr = changetype<usize>(url_buf);
  let url_len = url_buf.byteLength;

  let method_buf = String.UTF8.encode(method);
  let method_ptr = changetype<usize>(method_buf);
  let method_len = method_buf.byteLength;

  let req_headers_buf = String.UTF8.encode(headers);
  let req_headers_ptr = changetype<usize>(req_headers_buf);
  let req_headers_len = req_headers_buf.byteLength;

  let req_body_ptr = changetype<usize>(body);
  let req_body_len = body.byteLength;

  let status_code_ptr = memory.data(8);
  let res_handle_ptr = memory.data(8);

  let err = raw.req(
    url_ptr,
    url_len,
    method_ptr,
    method_len,
    req_headers_ptr,
    req_headers_len,
    req_body_ptr,
    req_body_len,
    status_code_ptr,
    res_handle_ptr
  );

  if (err != 0) {
    // Based on the error code, read and log the error.
    Console.log("ERROR CODE: " + err.toString());
    Console.log("ERROR MESSAGE: " + errorToHumanReadableMessage(err));
    abort();
  }

  let status = load<usize>(status_code_ptr) as u16;
  let handle = load<usize>(res_handle_ptr) as u32;

  return new Response(status, handle);
}

/** Transform the header map into a string. */
function headersToString(headers: Map<string, string>): string {
  let res = "";
  let keys = headers.keys() as string[];
  let values = headers.values() as string[];
  for (let index = 0, len = keys.length; index < len; ++index) {
    res += keys[index] + ":" + values[index] + "\n";
  }
  return res;
}

/** Transform the string representation of the headers into a map. */
function stringToHeaderMap(headersStr: string): Map<string, string> {
  let res = new Map<string, string>();
  let parts = headersStr.split("\n");
  // the result of the split contains an empty part as well
  for (let index = 0, len = parts.length - 1; index < len; index++) {
    let p = parts[index].split(":");
    res.set(p[0], p[1]);
  }

  return res;
}

/** The standard HTTP methods. */
export enum Method {
  GET,
  HEAD,
  POST,
  PUT,
  DELETE,
  CONNECT,
  OPTIONS,
  TRACE,
  PATCH,
}

/** Return the string representation of the HTTP method. */
function methodEnumToString(m: Method): string {
  switch (m) {
    case Method.GET:
      return "GET";
    case Method.HEAD:
      return "HEAD";
    case Method.POST:
      return "POST";
    case Method.PUT:
      return "PUT";
    case Method.DELETE:
      return "DELET";
    case Method.CONNECT:
      return "CONNECT";
    case Method.OPTIONS:
      return "OPTIONS";
    case Method.TRACE:
      return "TRACE";
    case Method.PATCH:
      return "PATCH";

    default:
      return "";
  }
}

function errorToHumanReadableMessage(e: u32): string {
  switch (e) {
    case 1:
      return "Invalid WASI HTTP handle.";
    case 2:
      return "Memory not found.";
    case 3:
      return "Memory access error.";
    case 4:
      return "Buffer too small";
    case 5:
      return "Header not found.";
    case 6:
      return "UTF-8 error.";
    case 7:
      return "Destination URL not allowed.";
    case 8:
      return "Invalid HTTP method.";
    case 9:
      return "Invalid encoding.";
    case 10:
      return "Invalid URL.";
    case 11:
      return "Unable to send HTTP request.";
    case 12:
      return "Runtime error.";
    case 13:
      return "Too many sessions.";

    default:
      return "Unknown error.";
  }
}

/** The standard HTTP status codes. */
export enum StatusCode {
  CONTINUE = 100,
  SWITCHING_PROTOCOL = 101,
  PROCESSING = 102,
  EARLY_HINTS = 103,

  OK = 200,
  CREATED = 201,
  ACCEPTED = 202,
  NON_AUTHORITATIVE_INFORMATION = 203,
  NO_CONTENT = 204,
  RESET_CONTENT = 205,
  PARTIAL_CONTENT = 206,
  MULTI_STATUS = 207,
  ALREADY_REPORTED = 208,
  IM_USED = 226,

  MULTIPLE_CHOICE = 300,
  MOVED_PERMANENTLY = 301,
  FOUND = 302,
  SEE_OTHER = 303,
  NOT_MODIFIED = 304,
  USE_PROXY = 305,
  UNUSED = 306,
  TEMPORARY_REDIRECT = 307,
  PERMANENT_REDIRECT = 308,

  BAD_REQUEST = 400,
  UNAUTHORIZED = 401,
  PAYMENT_REQUIRED = 402,
  FORBIDDEN = 403,
  NOT_FOUND = 404,
  METHOD_NOT_ALLOWED = 405,
  NOT_ACCEPTABLE = 406,
  PROXY_AUTHENTICATION_REQUIRED = 407,
  REQUEST_TIMEOUT = 408,
  CONFLICT = 409,
  GONE = 410,
  LENGTH_REQUIRED = 411,
  PRECONDITION_FAILED = 412,
  PAYLOAD_TOO_LARGE = 413,
  URI_TOO_LONG = 414,
  UNSUPPORTED_MEDIA_TYPE = 415,
  RANGE_NOT_SATISFIABLE = 416,
  EXPECTATION_FAILED = 417,
  IM_A_TEAPOT = 418,
  MISDIRECTED_REQUEST = 421,
  UNPROCESSABLE_ENTITY = 422,
  LOCKED = 423,
  FAILED_DEPENDENCY = 424,
  TOO_EARLY = 425,
  UPGRADE_REQUIRED = 426,
  PRECONDITION_REQURIED = 428,
  TOO_MANY_REQUESTS = 429,
  REQUEST_HEADER_FIELDS_TOO_LARGE = 431,
  UNAVAILABLE_FOR_LEGAL_REASONS = 451,

  INTERNAL_SERVER_ERROR = 500,
  NOT_IMPLELENTED = 501,
  BAD_GATEWAY = 502,
  SERVICE_UNAVAILABLE = 503,
  GATEWAY_TIMEOUT = 504,
  HTTP_VERSION_NOT_SUPPORTED = 505,
  VARIANT_ALSO_NEGOTIATES = 506,
  INSUFFICIENT_STORAGE = 507,
  LOOP_DETECTED = 508,
  NOT_EXTENDED = 510,
  NETWORK_AUTHENTICATION_REQUIRED = 511,
}
