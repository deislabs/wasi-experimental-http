// required to use --abort=as-wasi
// @ts-ignore
import * as wasi from "as-wasi";

export function _start(): void {
    let body = String.UTF8.encode("testing the body");
    let res = new RequestBuilder("https://postman-echo.com/post")
        .header("a", "b")
        .header("c", "d")
        .header("Content-Type", "text/plain")
        .method("POST")
        .body(body)
        .send();
    wasi.Console.log(res.status.toString())
    wasi.Console.log(res.headers);
    let result = String.UTF8.decode(res.body);
    wasi.Console.log(result);
}

export function request(req: Request): Response {
    return _request(req.url, req.method, headersToString(req.headers), req.body);
}

export class Response {
    public status: u16;
    public headers: string;
    public body: ArrayBuffer;

    constructor(status: u16, headers: string, body: ArrayBuffer) {
        this.status = status;
        this.headers = headers;
        this.body = body;
    }
}

export class Request {
    public url: string;
    public method: string;
    public headers: Map<string, string>;
    public body: ArrayBuffer;

    constructor(
        url: string,
        method: string = "GET",
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

    public method(m: string): RequestBuilder {
        this.request.method = m;
        return this;
    }

    public header(key: string, value: string): RequestBuilder {
        this.request.headers.set(key, value);
        return this;
    }

    public body(b: ArrayBuffer): RequestBuilder {
        this.request.body = b;
        return this;
    }

    public send(): Response {
        return _request(
            this.request.url,
            this.request.method,
            headersToString(this.request.headers),
            this.request.body
        );
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

function _request(
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
        // TODO
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

// Transform the header map into a string
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
        wasi.Console.log(res);
        return res;
    }

// Allocate memory for a new byte array of
// size `len` and return the offset into
// the module's linear memory to the start
// of the block.
export function alloc(len: i32): usize {
    let buf = new ArrayBuffer(len);
    return changetype<usize>(buf);
}
