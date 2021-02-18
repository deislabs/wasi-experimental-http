use anyhow::Error;
use http::{self, Request, Response};

pub fn request<T: Sized>(req: Request<T>) -> Result<Response<Vec<u8>>, Error> {
    let url = req.uri().to_string();
    let body = unsafe { raw_request(&url) };
    Ok(Response::new(body))
}

/// Transform the Rust `String` representing the URL into a pointer and length,
/// call the runtime's `wasi_experimental_http::req`, read the response body
/// from the memory and return it as a Rust `String`.
unsafe fn raw_request(url: &String) -> Vec<u8> {
    let url_len_ptr = &(url.len() as u32) as *const u32;
    let url_ptr = url.as_bytes().as_ptr() as *mut u8;
    let bytes_written_ptr = raw_ptr();
    let body_ptr = req(url_ptr, url_len_ptr, bytes_written_ptr as *mut u32);
    let bytes_written = *bytes_written_ptr as usize;
    Vec::from_raw_parts(body_ptr, bytes_written, bytes_written)
    // std::str::from_utf8(&ret_bytes).unwrap().to_string()
}

/// Import `wasi_experimental_http` from the runtime.
#[link(wasm_import_module = "wasi_experimental_http")]
extern "C" {
    pub fn req(url_ptr: *const u8, url_len_ptr: *const u32, bytes_written_ptr: *mut u32)
        -> *mut u8;
}

/// Allocate memory into the module's linear memory
/// and return the offset to the start of the block.
#[no_mangle]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(len);
    let ptr = buf.as_mut_ptr();

    std::mem::forget(buf);
    return ptr;
}

/// Get a raw pointer to a `u32` where the runtime can write the
/// number of bytes written.
unsafe fn raw_ptr() -> *const u32 {
    let x: Box<u32> = Box::new(0);
    let ptr: *const u32 = &*x;
    // TODO
    // We need to ensure no memory is leaked by doing this.
    std::mem::forget(x);
    ptr
}
