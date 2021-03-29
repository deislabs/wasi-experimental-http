use anyhow::Error;
use bytes::Bytes;
use http::{self, header::HeaderName, HeaderMap, HeaderValue, Request, StatusCode};
use std::str::FromStr;

/// HTTP errors
#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("Invalid handle")]
    InvalidHandle,
    #[error("Memory not found")]
    MemoryNotFound,
    #[error("Memory access error")]
    MemoryAccessError,
    #[error("Buffer too small")]
    BufferTooSmall,
    #[error("Header not found")]
    HeaderNotFound,
    #[error("UTF-8 error")]
    UTF8Error,
    #[error("Destination not allowed")]
    DestinationNotAllowed,
    #[error("Invalid method")]
    InvalidMethod,
    #[error("Invalid encoding")]
    InvalidEncoding,
    #[error("Invalid URL")]
    InvalidUrl,
    #[error("HTTP error")]
    RequestError,
    #[error("Runtime error")]
    RuntimeError,
    #[error("Too many sessions")]
    TooManySessions,
}

fn raw_err_check(e: u32) -> Result<(), HttpError> {
    match e {
        0 => Ok(()),
        1 => Err(HttpError::InvalidHandle),
        2 => Err(HttpError::MemoryNotFound),
        3 => Err(HttpError::MemoryAccessError),
        4 => Err(HttpError::BufferTooSmall),
        5 => Err(HttpError::HeaderNotFound),
        6 => Err(HttpError::UTF8Error),
        7 => Err(HttpError::DestinationNotAllowed),
        8 => Err(HttpError::InvalidMethod),
        9 => Err(HttpError::InvalidEncoding),
        10 => Err(HttpError::InvalidUrl),
        11 => Err(HttpError::RequestError),
        12 => Err(HttpError::RuntimeError),
        13 => Err(HttpError::TooManySessions),
        _ => unreachable!(),
    }
}

type Handle = u32;

/// A HTTP response.
pub struct Response {
    handle: Handle,
    pub status_code: StatusCode,
}

impl Drop for Response {
    fn drop(&mut self) {
        unsafe { raw::close(self.handle) };
    }
}

impl Response {
    /// Read a response body in a streaming fashion.
    /// `buf` is an arbitrary large buffer, that may be partially filled after each call.
    /// The function returns the actual number of bytes that were written, and `0`
    /// when the end of the stream has been reached.
    pub fn body_read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let mut read: usize = 0;
        let ret = unsafe { raw::body_read(self.handle, buf.as_mut_ptr(), buf.len(), &mut read) };
        raw_err_check(ret)?;
        Ok(read)
    }

    /// Read the entire body until the end of the stream.
    pub fn body_read_all(&mut self) -> Result<Vec<u8>, Error> {
        let mut chunk = [0u8; 4096];
        let mut v = vec![];
        loop {
            let read = self.body_read(&mut chunk)?;
            if read == 0 {
                return Ok(v);
            }
            v.extend_from_slice(&chunk[0..read]);
        }
    }

    /// Get the value of the `name` header.
    /// Returns `HttpError::HeaderNotFound` if no such header was found.
    pub fn header_get(&self, name: &str) -> Result<String, Error> {
        let mut capacity = 4096;
        loop {
            let mut written: usize = 0;
            let mut buf = vec![0u8; capacity];
            let ret = unsafe {
                raw::header_get(
                    self.handle,
                    name.as_ptr(),
                    name.len(),
                    buf.as_mut_ptr(),
                    buf.len(),
                    &mut written,
                )
            };
            match raw_err_check(ret) {
                Ok(()) => {
                    buf.truncate(written);
                    return Ok(String::from_utf8(buf)?);
                }
                Err(HttpError::BufferTooSmall) => {
                    capacity *= 2;
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}

/// Send a HTTP request.
/// The function returns a `Response` object, that includes the status,
/// as well as methods to access the headers and the body.
#[tracing::instrument]
pub fn request(req: Request<Option<Bytes>>) -> Result<Response, Error> {
    let url = req.uri().to_string();
    tracing::debug!(%url, headers = ?req.headers(), "performing http request using wasmtime function");

    let headers = header_map_to_string(req.headers())?;
    let method = req.method().as_str();
    let body = match req.body() {
        None => Default::default(),
        Some(body) => body.as_ref(),
    };
    let mut status_code: u16 = 0;
    let mut handle: Handle = 0;
    let ret = unsafe {
        raw::req(
            url.as_ptr(),
            url.len(),
            method.as_ptr(),
            method.len(),
            headers.as_ptr(),
            headers.len(),
            body.as_ptr(),
            body.len(),
            &mut status_code,
            &mut handle,
        )
    };
    raw_err_check(ret)?;
    Ok(Response {
        handle,
        status_code: StatusCode::from_u16(status_code)?,
    })
}

/// Encode a header map as a string.
pub fn header_map_to_string(hm: &HeaderMap) -> Result<String, Error> {
    let mut res = String::new();
    for (name, value) in hm
        .iter()
        .map(|(name, value)| (name.as_str(), std::str::from_utf8(value.as_bytes())))
    {
        let value = value?;
        anyhow::ensure!(
            !name
                .chars()
                .any(|x| x.is_control() || "(),/:;<=>?@[\\]{}".contains(x)),
            "Invalid header name"
        );
        anyhow::ensure!(
            !value.chars().any(|x| x.is_control()),
            "Invalid header value"
        );
        res.push_str(&format!("{}:{}\n", name, value));
    }
    Ok(res)
}

/// Decode a header map from a string.
pub fn string_to_header_map(s: &str) -> Result<HeaderMap, Error> {
    let mut headers = HeaderMap::new();
    for entry in s.lines() {
        let mut parts = entry.splitn(2, ':');
        let k = parts.next().ok_or(anyhow::format_err!(
            "Invalid serialized header: [{}]",
            entry
        ))?;
        let v = parts.next().unwrap();
        headers.insert(HeaderName::from_str(k)?, HeaderValue::from_str(v)?);
    }
    Ok(headers)
}

mod raw {
    /// Import `wasi_experimental_http` from the runtime.
    #[link(wasm_import_module = "wasi_experimental_http")]
    extern "C" {
        pub fn req(
            url_ptr: *const u8,
            url_len_ptr: usize,
            method_ptr: *const u8,
            method_len_ptr: usize,
            req_headers_ptr: *const u8,
            req_headers_len_ptr: usize,
            req_body_ptr: *const u8,
            req_body_len_ptr: usize,
            status_code_ptr: *mut u16,
            res_handle_ptr: *mut u32,
        ) -> u32;

        pub fn close(handle: u32) -> u32;

        pub fn header_get(
            handle: u32,
            name_ptr: *const u8,
            name_len: usize,
            value_ptr: *mut u8,
            value_len: usize,
            value_written_ptr: *mut usize,
        ) -> u32;

        pub fn body_read(
            handle: u32,
            buf_ptr: *mut u8,
            buf_len: usize,
            but_read_ptr: *mut usize,
        ) -> u32;
    }
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
        assert_eq!(
            "custom-header:custom-value\ncustom-header2:custom-value2\n",
            str
        );
    }
}
