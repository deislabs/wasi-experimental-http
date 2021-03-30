// required to use --abort=as-wasi
// @ts-ignore
import { Console } from "as-wasi";

/** Send an HTTP request and return an HTTP response.
 *
 * It is recommended to use the `RequestBuilder` helper class.
 */
export function request(req: Request): Response {
    return raw_request(req.url, methodEnumToString(req.method), headersToString(req.headers), req.body);
}

/** An HTTP response. */
export class Response {
    /** The HTTP response status code. */
    public status: StatusCode;

    /** The response handle */
    handle: u32;

    constructor(status: u16, handle: u32) {
        this.status = status;
        this.handle = handle;
    }

    public bodyRead(buffer: ArrayBuffer): usize {
        let buf_read_ptr = memory.data(8);
        if (body_read(this.handle, changetype<usize>(buffer), buffer.byteLength, buf_read_ptr) != 0) {
            return 0;
        }
        return load<isize>(buf_read_ptr);
    }

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

    public headerGet(name: string): string {
        let name_buf = String.UTF8.encode(name);
        let name_ptr = changetype<usize>(name_buf);
        let name_len = name_buf.byteLength;

        let value_buf = new Uint8Array(4096);
        let value_buf_ptr = changetype<usize>(value_buf.buffer);
        let value_buf_len = value_buf.byteLength;
        let value_len_ptr = memory.data(8);

        if (header_get(this.handle, name_ptr, name_len, value_buf_ptr, value_buf_len, value_len_ptr) != 0) {
            return "";
        }
        let value = value_buf.subarray(0, load<u32>(value_len_ptr));
        return String.UTF8.decode(value.buffer);
    }

    public close(): void {
        close(this.handle);
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

/** Helper class for creating and sending an HTTP request.
 * ```
    let body = String.UTF8.encode("testing the body");
    let res = new RequestBuilder("https://SOME-URL")
        .header("Content-Type", "text/plain")
        .method(Method.POST)
        .body(body)
        .send();
    Console.log(res.status.toString())
    Console.log(res.headers);
    let result = String.UTF8.decode(res.bodyReadAll().buffer);
    Console.log(result);
 * ```
*/
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

// @ts-ignore: decorator
@external("wasi_experimental_http", "req")
@unsafe declare function req(
    url_ptr: usize,
    url_len_ptr: usize,
    method_ptr: usize,
    method_len_ptr: usize,
    req_headers_ptr: usize,
    req_headers_len_ptr: usize,
    req_body_ptr: usize,
    req_body_len_ptr: usize,
    status_code_ptr: usize,
    res_handle_ptr: usize,
): u32;

// @ts-ignore: decorator
@external("wasi_experimental_http", "close")
@unsafe declare function close(
    handle: usize
): u32;

// @ts-ignore: decorator
@external("wasi_experimental_http", "header_get")
@unsafe declare function header_get(
    handle: usize,
    name_ptr: usize,
    name_len: usize,
    value_ptr: usize,
    value_len: usize,
    value_written_ptr: usize,
): u32;

// @ts-ignore: decorator
@external("wasi_experimental_http", "body_read")
@unsafe declare function body_read(
    handle: usize,
    buf_ptr: usize,
    buf_len: usize,
    but_read_ptr: usize
): u32;

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

    let err = req(
        url_ptr,
        url_len,
        method_ptr,
        method_len,
        req_headers_ptr,
        req_headers_len,
        req_body_ptr,
        req_body_len,
        status_code_ptr,
        res_handle_ptr,
    );

    if (err != 0) {
        // Based on the error code, read and log the error.
        Console.log("ERROR CODE: " + err.toString());

        // An error code was written. Read it, then abort.
        Console.log("Runtime error: " + err.toString());
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
        res += keys[index] + ":" + values[index] + '\n';
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
    PATCH
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
    NETWORK_AUTHENTICATION_REQUIRED = 511
}
