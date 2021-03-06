// required to use --abort=as-wasi
// @ts-ignore
import * as wasi from "as-wasi";

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
    /** The HTTP response headers.
     *
     * TODO
     *
     * For now, the response headers are represented
     * as a string, which makes it difficult to actually
     * use by a consumer. This should be represented as a
     * `Map<string, string>`.
     */
    public headers: string;
    /** The response body as a byte array.
     *
     * It should be decoded by the consumer accordingly.
     * If expecting a UTF string, use the built-in functions
     * to decode.
     */
    public body: ArrayBuffer;

    constructor(status: u16, headers: string, body: ArrayBuffer) {
        this.status = status;
        this.headers = headers;
        this.body = body;
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
    wasi.Console.log(res.status.toString())
    wasi.Console.log(res.headers);
    let result = String.UTF8.decode(res.body);
    wasi.Console.log(result);
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
        req_body_ptr: usize,
        req_body_len_ptr: usize,
        heders_ptr: usize,
        headers_len_ptr: usize,
        body_res_ptr: usize,
        body_written_ptr: usize,
        headers_written_ptr: usize,
        headers_res_ptr: usize,
        status_code_ptr: usize,
        err_ptr: usize,
        err_len_ptr: usize
    ): u32;

function raw_request(
        url: string,
        method: string,
        headers: string,
        body: ArrayBuffer
    ): Response {

    let url_buf = String.UTF8.encode(url);
    let url_len_ptr = memory.data(8);
    store<usize>(url_len_ptr, url_buf.byteLength);
    let url_ptr = changetype<usize>(url_buf);

    let method_buf = String.UTF8.encode(method);
    let method_len_ptr = memory.data(8);
    store<usize>(method_len_ptr, method_buf.byteLength);
    let method_ptr = changetype<usize>(method_buf);

    let headers_buf = String.UTF8.encode(headers);
    let headers_len_ptr = memory.data(8);
    store<usize>(headers_len_ptr, headers_buf.byteLength);
    let headers_ptr = changetype<usize>(headers_buf);

    let req_body_ptr = changetype<usize>(body);
    let req_body_len_ptr = memory.data(8);
    store<usize>(req_body_len_ptr, body.byteLength);

    let body_res_ptr = memory.data(8);
    let body_written_ptr = memory.data(8);
    let headers_res_ptr = memory.data(8);
    let headers_written_ptr = memory.data(8);
    let status_code_ptr = memory.data(8);
    let err_ptr = memory.data(8);
    let err_len_ptr = memory.data(8);

    let err = req(
        url_ptr,
        url_len_ptr,
        method_ptr,
        method_len_ptr,
        req_body_ptr,
        req_body_len_ptr,
        headers_ptr,
        headers_len_ptr,
        body_res_ptr,
        body_written_ptr,
        headers_written_ptr,
        headers_res_ptr,
        status_code_ptr,
        err_ptr,
        err_len_ptr
    );

    if (err != 0) {
        // Based on the error code, read and log the error.
        wasi.Console.log("ERROR CODE: " + err.toString());

        // Error code 1 means no error message was written.
        if (err == 1) {
            wasi.Console.log("Runtime error: cannot find exorted alloc function or memory");
            abort();
        }

        // An error code was written. Read it, then abort.
        let err_len = load<usize>(err_len_ptr) as u32;
        let err_buf = new ArrayBuffer(err_len);
        memory.copy(changetype<usize>(err_buf), err_ptr, err_len);
        wasi.Console.log("Runtime error: " + String.UTF8.decode(err_buf));
        abort();
    }

    let status = load<usize>(status_code_ptr) as u16;

    let body_size = load<usize>(body_written_ptr) as u32;
    let body_res = new ArrayBuffer(body_size);
    memory.copy(changetype<usize>(body_res), load<usize>(body_res_ptr), body_size);

    let headers_length = load<usize>(headers_written_ptr) as u32;
    let headers_res_buf = new ArrayBuffer(headers_length);
    memory.copy(changetype<usize>(headers_res_buf), load<usize>(headers_res_ptr), headers_length);
    let headers_res = String.UTF8.decode(headers_res_buf);

    return new Response(status, headers_res, body_res);
}

/** Transform the header map into a string. */
function headersToString(headers: Map<string, string>): string {
    let res: string = "{";
    let keys = headers.keys() as Array<string>;
    let values = headers.values() as Array<string>;
    for (let index = 0; index < keys.length; ++index) {
        res += '"' + keys[index] + '"' + ":" + '"' + values[index] + '"';
        if (index != keys.length - 1) {
            res += ",";
        }
    }
    res += "}";

    return res;
}

/** The standard HTTP methods. */
export enum Method{
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
    switch(m) {
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

/** Allocate memory for a new byte array of size `len`
 * and return the offset into the module's linear memory
 * to the start of the block. */
export function alloc(len: i32): usize {
    let buf = new ArrayBuffer(len);
    return changetype<usize>(buf);
}
