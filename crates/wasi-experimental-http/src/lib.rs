use std::{collections::HashMap, str::FromStr};

use anyhow::Error;
use http::{self, header::HeaderName, HeaderMap, HeaderValue, Request, Response, StatusCode};

pub fn request(req: Request<Option<Vec<u8>>>) -> Result<Response<Vec<u8>>, Error> {
    let url = req.uri().to_string();
    let headers = header_map_to_string(req.headers())?;
    let (body, headers, status_code) =
        unsafe { raw_request(&url, req.method().to_string(), &headers, req.body())? };
    let mut res = Response::builder().status(StatusCode::from_u16(status_code)?);
    append_headers(
        res.headers_mut().unwrap(),
        std::str::from_utf8(&headers)?.to_string(),
    )?;

    Ok(res.body(body)?)
}

// TODO
// This is a very inneficient way of getting a String from a `HeaderMap`.
pub fn header_map_to_string(hm: &HeaderMap) -> Result<String, Error> {
    let mut res: HashMap<String, String> = HashMap::new();
    for (k, v) in hm.iter() {
        res.insert(k.to_string(), v.to_str()?.to_string());
    }
    let res = serde_json::to_value(&res)?.to_string();
    Ok(res)
}

// TODO
// This is a very inneficient way of getting a `HeaderMap` from a `String.
pub fn string_to_header_map(hm: String) -> Result<HeaderMap, Error> {
    let hm: HashMap<String, String> = serde_json::from_str(&hm)?;
    let mut headers = HeaderMap::new();
    for (k, v) in hm.iter() {
        headers.insert(HeaderName::from_str(k)?, HeaderValue::from_str(v)?);
    }
    Ok(headers)
}

fn append_headers(res_headers: &mut HeaderMap, source: String) -> Result<(), Error> {
    let hm: HashMap<String, String> = serde_json::from_str(&source)?;
    for (k, v) in hm.iter() {
        res_headers.insert(HeaderName::from_str(k)?, HeaderValue::from_str(v)?);
    }
    Ok(())
}

/// Transform the Rust `String` representing the URL into a pointer and length,
/// call the runtime's `wasi_experimental_http::req`, read the response
/// from the memory and return it as a Rust `String`.
unsafe fn raw_request(
    url: &String,
    method: String,
    headers: &String,
    body: &Option<Vec<u8>>,
) -> Result<(Vec<u8>, Vec<u8>, u16), Error> {
    let body = match body {
        Some(b) => b.clone(),
        None => Vec::new(),
    };

    let req_body_ptr = body.as_ptr() as *mut u32;
    let req_body_len_ptr = &(body.len() as u32) as *const u32;

    let method_len_ptr = &(method.len() as u32) as *const u32;
    let method_ptr = method.as_bytes().as_ptr() as *mut u32;

    let url_len_ptr = &(url.len() as u32) as *const u32;
    let url_ptr = url.as_bytes().as_ptr() as *mut u32;

    let headers_len_ptr = &(headers.len() as u32) as *const u32;
    let headers_ptr = headers.as_bytes().as_ptr() as *mut u32;

    let body_res_ptr = raw_ptr();
    let body_written_ptr = raw_ptr();

    let headers_written_ptr = raw_ptr();
    let headers_res_ptr = raw_ptr();

    let status_code_ptr = raw_ptr();

    let err_ptr = raw_ptr();
    let err_len_ptr = raw_ptr();

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
        err_len_ptr,
    );

    if err != 0 {
        println!("error code: {}", err);
        let bytes = Vec::from_raw_parts(
            *err_ptr as *mut u8,
            *err_len_ptr as usize,
            *err_len_ptr as usize,
        );
        let msg = std::str::from_utf8(bytes.as_slice())?.to_string();
        return Err(Error::msg(msg));
    };

    let bytes_written = *body_written_ptr as usize;
    let headers_written = *headers_written_ptr as usize;

    Ok((
        Vec::from_raw_parts(*body_res_ptr as *mut u8, bytes_written, bytes_written),
        Vec::from_raw_parts(
            *headers_res_ptr as *mut u8,
            headers_written,
            headers_written,
        ),
        *status_code_ptr as u16,
    ))
}

/// Import `wasi_experimental_http` from the runtime.
#[link(wasm_import_module = "wasi_experimental_http")]
extern "C" {
    fn req(
        url_ptr: *const u32,
        url_len_ptr: *const u32,
        method_ptr: *const u32,
        method_len_ptr: *const u32,
        req_body_ptr: *const u32,
        req_body_len_ptr: *const u32,
        headers_ptr: *const u32,
        headers_len_ptr: *const u32,
        body_res_ptr: *const u32,
        body_written_ptr: *const u32,
        headers_written_ptr: *const u32,
        headers_res_ptr: *const u32,
        status_code_ptr: *const u32,
        err_ptr: *const u32,
        err_len_ptr: *const u32,
    ) -> u32;
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

#[cfg(test)]
mod tests {
    use super::*;
    use http::{HeaderMap, HeaderValue};

    #[test]
    fn test_header_map_to_string() {
        let mut hm = HeaderMap::new();
        hm.insert("custom-header", HeaderValue::from_static("custom-value"));
        hm.insert("custom-header2", HeaderValue::from_static("custom-value2"));
        let str = header_map_to_string(&hm).unwrap();
        println!("{}", str);
    }

    #[test]
    fn test_string_to_header_map() {
        let headers = r#"{"custom-header":"custom-value","custom-header2":"custom-value2"}"#;
        let header_map = string_to_header_map(headers.to_string()).unwrap();
        println!("{:#?}", header_map);
    }
}
