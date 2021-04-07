use anyhow::Error;
use bytes::Bytes;
use http::{self, header::HeaderName, HeaderMap, HeaderValue, Request, StatusCode};
use std::str::FromStr;

#[allow(dead_code)]
#[allow(clippy::mut_from_ref)]
#[allow(clippy::clippy::too_many_arguments)]
pub(crate) mod raw;

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
    Utf8Error,
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
    #[error("Unknown WASI error")]
    UnknownError,
}

// TODO
//
// This error is not really used in the public API.
impl From<raw::Error> for HttpError {
    fn from(e: raw::Error) -> Self {
        match e {
            raw::Error::WasiError(errno) => match errno {
                1 => HttpError::InvalidHandle,
                2 => HttpError::MemoryNotFound,
                3 => HttpError::MemoryAccessError,
                4 => HttpError::BufferTooSmall,
                5 => HttpError::HeaderNotFound,
                6 => HttpError::Utf8Error,
                7 => HttpError::DestinationNotAllowed,
                8 => HttpError::InvalidMethod,
                9 => HttpError::InvalidEncoding,
                10 => HttpError::InvalidUrl,
                11 => HttpError::RequestError,
                12 => HttpError::RuntimeError,
                13 => HttpError::TooManySessions,

                _ => HttpError::UnknownError,
            },
        }
    }
}

/// An HTTP response
pub struct Response {
    handle: raw::ResponseHandle,
    pub status_code: StatusCode,
}

/// Automatically call `close` to remove the current handle
/// when the response object goes out of scope.
impl Drop for Response {
    fn drop(&mut self) {
        raw::close(self.handle).unwrap();
    }
}

impl Response {
    /// Read a response body in a streaming fashion.
    /// `buf` is an arbitrary large buffer, that may be partially filled after each call.
    /// The function returns the actual number of bytes that were written, and `0`
    /// when the end of the stream has been reached.
    pub fn body_read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let read = raw::body_read(self.handle, buf.as_mut_ptr(), buf.len())?;

        // raw_err_check(ret)?;
        Ok(read)
    }

    /// Read the entire body until the end of the stream.
    pub fn body_read_all(&mut self) -> Result<Vec<u8>, Error> {
        // TODO(@radu-matei)
        //
        // Do we want to have configurable chunk sizes?
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
    pub fn header_get(&self, name: String) -> Result<String, Error> {
        let mut name = name;

        // Set the initial capacity of the expected header value to 4 kilobytes.
        // If the response value size is larger, double the capacity and
        // attempt to read again, but only until reaching 64 kilobytes.
        //
        // This is to avoid a potentially malicious web server from returning a
        // response header that would make the guest allocate all of its possible
        // memory.
        // The maximum is set to 64 kilobytes, as it is usually the maximum value
        // known servers will allow until returning 413 Entity Too Large.
        let mut capacity = 4 * 1024;
        let max_capacity: usize = 64 * 1024;

        loop {
            let mut buf = vec![0u8; capacity];
            match raw::header_get(
                self.handle,
                name.as_mut_ptr(),
                name.len(),
                buf.as_mut_ptr(),
                buf.len(),
            ) {
                Ok(written) => {
                    buf.truncate(written);
                    return Ok(String::from_utf8(buf)?);
                }
                Err(e) => match Into::<HttpError>::into(e) {
                    HttpError::BufferTooSmall => {
                        if capacity < max_capacity {
                            capacity *= 2;
                            continue;
                        } else {
                            return Err(e.into());
                        }
                    }
                    _ => return Err(e.into()),
                },
            };
        }
    }
}

/// Send an HTTP request.
/// The function returns a `Response` object, that includes the status,
/// as well as methods to access the headers and the body.
#[tracing::instrument]
pub fn request(req: Request<Option<Bytes>>) -> Result<Response, Error> {
    let mut url = req.uri().to_string();
    tracing::debug!(%url, headers = ?req.headers(), "performing http request using wasmtime function");

    let mut headers = header_map_to_string(req.headers())?;
    let mut method = req.method().as_str().to_string();
    let body = match req.body() {
        None => Default::default(),
        Some(body) => body.as_ref(),
    };
    let (status_code, handle) = raw::req(
        url.as_mut_ptr(),
        url.len(),
        method.as_mut_ptr(),
        method.len(),
        headers.as_mut_ptr(),
        headers.len(),
        body.as_ptr(),
        body.len(),
    )?;

    // raw_err_check(ret)?;
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
        #[allow(clippy::clippy::clippy::or_fun_call)]
        let k = parts.next().ok_or(anyhow::format_err!(
            "Invalid serialized header: [{}]",
            entry
        ))?;
        let v = parts.next().unwrap();
        headers.insert(HeaderName::from_str(k)?, HeaderValue::from_str(v)?);
    }
    Ok(headers)
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
