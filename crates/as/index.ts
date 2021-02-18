// required to use --abort=as-wasi
// @ts-ignore
import * as wasi from "as-wasi";

// @ts-ignore: decorator
@external("wasi_experimental_http", "req")
@unsafe export declare function req(url_ptr: usize, url_len_ptr: usize, bytes_written_ptr: usize): usize;

export function _start(): void {
    let res = request("https://api.brigade.sh/healthz");
    wasi.Console.log(res);
}

export function request(url: string): string {
    let url_buf = String.UTF8.encode(url);
    let url_len = url_buf.byteLength;
    let url_len_ptr = memory.data(8);
    store<usize>(url_len_ptr, url_len);
    let url_ptr = changetype<usize>(url_buf);
    let bytes_written_ptr = memory.data(8);
    let body_ptr = req(url_ptr, url_len_ptr, bytes_written_ptr);
    // @ts-ignore: cast
    return String.UTF8.decodeUnsafe(body_ptr, load<usize>(bytes_written_ptr));
}

// Allocate memory for a new byte array of
// size `len` and return the offset into
// the module's linear memory to the start
// of the block.
export function alloc(len: i32): usize {
    // create a new AssemblyScript byte array
    let buf = new Array<u8>(len);
    let buf_ptr = memory.data(8);
    // create a pointer to the byte array and
    // return it
    store<Array<u8>>(buf_ptr, buf);
    return buf_ptr;
  }
