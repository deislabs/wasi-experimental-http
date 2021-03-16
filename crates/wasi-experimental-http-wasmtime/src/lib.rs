use anyhow::Error;
use bytes::Bytes;
use futures::executor::block_on;
use http::{HeaderMap, HeaderValue};
use reqwest::{Client, Method};
use std::str::FromStr;
use tokio::runtime::Handle;
use url::Url;
use wasmtime::*;

const ALLOC_FN: &str = "alloc";
const MEMORY: &str = "memory";

pub fn link_http(linker: &mut Linker, allowed_domains: Option<Vec<String>>) -> Result<(), Error> {
    linker.func(
        "wasi_experimental_http",
        "req",
        move |caller: Caller<'_>,
              url_ptr: u32,
              url_len: u32,
              method_ptr: u32,
              method_len: u32,
              req_body: u32,
              req_body_len: u32,
              headers_ptr: u32,
              headers_len: u32,
              body_res_ptr: u32,
              body_written_ptr: u32,
              headers_res_ptr: u32,
              headers_written_ptr: u32,
              status_code_ptr: u32,
              err_ptr: u32,
              err_written_ptr: u32|
              -> u32 {
            let span = tracing::trace_span!("req");
            let _enter = span.enter();
            // Get the module's memory and allocation function.
            // If either is not found, the runtime cannot write any response
            // data, so the execution cannot continue.
            tracing::trace!(export = MEMORY, "getting export");
            let memory = match caller.get_export(MEMORY) {
                Some(Extern::Memory(mem)) => mem,
                _ => {
                    return err(
                        "cannot find memory".to_string(),
                        None,
                        None,
                        err_ptr,
                        err_written_ptr,
                        1,
                    )
                }
            };
            tracing::trace!(export = ALLOC_FN, "getting export");
            let alloc = match caller.get_export(ALLOC_FN) {
                Some(Extern::Func(func)) => func,
                _ => {
                    return err(
                        "cannot find alloc function".to_string(),
                        None,
                        None,
                        err_ptr,
                        err_written_ptr,
                        1,
                    )
                }
            };

            // Get the URL, headers, method, and request body from the module's memory.
            let (url, headers, method, req_body) = match unsafe {
                http_parts_from_memory(
                    &memory,
                    url_ptr,
                    url_len,
                    method_ptr,
                    method_len,
                    req_body,
                    req_body_len,
                    headers_ptr,
                    headers_len,
                )
            } {
                Ok(r) => (r.0, r.1, r.2, r.3),
                Err(_) => {
                    return err(
                        "cannot get HTTP parts from memory".to_string(),
                        Some(&memory),
                        Some(&alloc),
                        err_ptr,
                        err_written_ptr,
                        4,
                    )
                }
            };

            match is_allowed(&url, allowed_domains.as_ref()) {
                Ok(e) => match e {
                    true => {}
                    false => {
                        return err(
                            format!(
                            "URL {} not allowed because domain or subdomain not in allowed list",
                            url
                        ),
                            Some(&memory),
                            Some(&alloc),
                            err_ptr,
                            err_written_ptr,
                            4,
                        )
                    }
                },
                Err(e) => {
                    return err(
                        e.to_string(),
                        Some(&memory),
                        Some(&alloc),
                        err_ptr,
                        err_written_ptr,
                        4,
                    )
                }
            };

            let (status, headers, body) = match request(url, headers, method, req_body) {
                Ok(r) => (r.0, r.1, r.2),
                Err(e) => {
                    return err(
                        e.to_string(),
                        Some(&memory),
                        Some(&alloc),
                        err_ptr,
                        err_written_ptr,
                        3,
                    )
                }
            };
            tracing::debug!(
                status,
                ?headers,
                body_len = body.as_ref().len(),
                "got HTTP response, writing back to memory"
            );

            // Write the HTTP response back to the module's memory.
            unsafe {
                match write_http_response_to_memory(
                    status,
                    headers,
                    body,
                    memory.clone(),
                    alloc.clone(),
                    headers_written_ptr,
                    headers_res_ptr,
                    body_res_ptr,
                    status_code_ptr,
                    body_written_ptr,
                ) {
                    Ok(_) => 0,
                    Err(e) => err(
                        e.to_string(),
                        Some(&memory),
                        Some(&alloc),
                        err_ptr,
                        err_written_ptr,
                        3,
                    ),
                }
            }
        },
    )?;

    Ok(())
}

#[tracing::instrument]
fn request(
    url: String,
    headers: HeaderMap,
    method: Method,
    body: Vec<u8>,
) -> Result<(u16, HeaderMap<HeaderValue>, Bytes), Error> {
    tracing::debug!(
        %url,
        ?headers,
        ?method,
        body_len = body.len(),
        "performing request"
    );
    match Handle::try_current() {
        Ok(r) => {
            // If running in a Tokio runtime, spawn a new blocking executor
            // that will send the HTTP request, and block on its execution.
            // This attempts to avoid any deadlocks from other operations
            // already executing on the same executor (compared with just
            // blocking on the current one).
            //
            // This should only be a temporary workaround, until we take
            // advantage of async functions in Wasmtime.
            tracing::trace!("tokio runtime available, spawning request on tokio thread");
            block_on(r.spawn_blocking(move || {
                let client = Client::builder().build().unwrap();
                let res = block_on(
                    client
                        .request(method, &url)
                        .headers(headers)
                        .body(body)
                        .send(),
                )?;

                return Ok((
                    res.status().as_u16(),
                    res.headers().clone(),
                    block_on(res.bytes())?,
                ));
            }))?
        }
        Err(_) => {
            tracing::trace!("no tokio runtime available, using blocking request");
            let res = reqwest::blocking::Client::new()
                .request(method, &url)
                .headers(headers)
                .body(body)
                .send()?;
            return Ok((res.status().as_u16(), res.headers().clone(), res.bytes()?));
        }
    }
}

/// Get the URL, method, request body, and headers from the
/// module's memory.
#[allow(clippy::too_many_arguments)]
unsafe fn http_parts_from_memory(
    memory: &Memory,
    url_ptr: u32,
    url_len: u32,
    method_ptr: u32,
    method_len: u32,
    req_body_ptr: u32,
    req_body_len: u32,
    headers_ptr: u32,
    headers_len: u32,
) -> Result<(String, HeaderMap, Method, Vec<u8>), Error> {
    let url = string_from_memory(&memory, url_ptr, url_len)?;
    let headers = string_from_memory(&memory, headers_ptr, headers_len)?;
    let headers = wasi_experimental_http::string_to_header_map(headers)?;
    let method = string_from_memory(&memory, method_ptr, method_len)?;
    let method = Method::from_str(&method)?;
    let req_body = vec_from_memory(&memory, req_body_ptr, req_body_len);

    Ok((url, headers, method, req_body))
}

/// Write the response data to the module's memory.
#[allow(clippy::too_many_arguments)]
unsafe fn write_http_response_to_memory(
    status: u16,
    headers: HeaderMap,
    res: Bytes,
    memory: Memory,
    alloc: Func,
    headers_written_ptr: u32,
    headers_res_ptr: u32,
    body_res_ptr: u32,
    status_code_ptr: u32,
    body_written_ptr: u32,
) -> Result<(), Error> {
    let hs = wasi_experimental_http::header_map_to_string(&headers)?;
    // Write the response headers.
    write(&hs, headers_res_ptr, headers_written_ptr, &memory, &alloc)?;

    // Write the status code pointer.
    let status_tmp_ptr = memory.data_ptr().offset(status_code_ptr as isize) as *mut u16;
    *status_tmp_ptr = status;

    // Write the response body.
    write(&res, body_res_ptr, body_written_ptr, &memory, &alloc)?;

    Ok(())
}

fn is_allowed(url: &str, allowed_domains: Option<&Vec<String>>) -> Result<bool, Error> {
    let url_host = Url::parse(url)?.host_str().unwrap().to_owned();
    match allowed_domains {
        Some(domains) => {
            let allowed: Result<Vec<_>, _> = domains.iter().map(|d| Url::parse(d)).collect();
            let allowed = allowed?;
            let a: Vec<&str> = allowed.iter().map(|u| u.host_str().unwrap()).collect();
            Ok(a.contains(&url_host.as_str()))
        }
        None => Ok(true),
    }
}

/// Write error details into the module's memory and return.
fn err(
    msg: String,
    memory: Option<&Memory>,
    alloc: Option<&Func>,
    err_ptr: u32,
    err_len_ptr: u32,
    err_code: u32,
) -> u32 {
    let memory = match memory {
        Some(m) => m,
        None => return err_code,
    };
    let alloc = match alloc {
        Some(a) => a,
        None => return err_code,
    };
    let _ = write(&msg, err_ptr, err_len_ptr, memory, alloc);
    err_code
}

/// Read a byte array from the instance's `memory`  of length `len_ptr`
/// starting at offset `data_ptr`
unsafe fn data_from_memory(memory: &Memory, data_ptr: u32, len: u32) -> (Option<&[u8]>, u32) {
    println!("wasi_experimental_http::data_from_memory:: length: {}", len);

    let data = memory
        .data_unchecked()
        .get(data_ptr as u32 as usize..)
        .and_then(|arr| arr.get(..len as u32 as usize));

    (data, len)
}

/// Get a `Vec<u8>` from the module's memory.
unsafe fn vec_from_memory(memory: &Memory, data_ptr: u32, len_ptr: u32) -> Vec<u8> {
    let (data, _) = data_from_memory(&memory, data_ptr, len_ptr);
    // Normally, this should never panic.
    data.unwrap_or_default().to_vec()
}

/// Read a string from the instance's `memory`  of length `len`
/// starting at offset `data_ptr`
unsafe fn string_from_memory(
    memory: &Memory,
    data_ptr: u32,
    len: u32,
) -> Result<String, anyhow::Error> {
    let (data, _) = data_from_memory(&memory, data_ptr, len);
    let str = match data {
        Some(data) => match std::str::from_utf8(data) {
            Ok(s) => s,
            Err(_) => return Err(anyhow::Error::msg("invalid utf-8")),
        },
        None => return Err(anyhow::Error::msg("pointer/length out of bounds")),
    };

    Ok(String::from(str))
}

/// Write a byte array into the instance's linear memory
/// and return the offset relative to the module's memory.
fn write(
    data: impl AsRef<[u8]>,
    ptr: u32,
    bytes_written_ptr: u32,
    memory: &Memory,
    alloc: &Func,
) -> Result<(), Error> {
    let bytes = data.as_ref();
    let alloc_result = alloc.call(&[Val::from(bytes.len() as i32)])?;
    let guest_ptr_offset = match alloc_result
        .get(0)
        .expect("expected the result of the allocation to have one value")
    {
        Val::I32(val) => *val as isize,
        _ => return Err(Error::msg("guest pointer must be Val::I32")),
    };
    unsafe {
        let raw = memory.data_ptr().offset(guest_ptr_offset);
        raw.copy_from(bytes.as_ptr(), bytes.len());

        // Get the offsite to `written` in the module's memory and set its value
        // to the number of body bytes written.
        let written_ptr = memory.data_ptr().offset(bytes_written_ptr as isize) as *mut u32;
        *written_ptr = bytes.len() as u32;
        println!(
            "wasi_experimental_http::write_guest_memory:: written {} bytes",
            *written_ptr
        );

        // Write result pointer.
        let res_ptr = memory.data_ptr().offset(ptr as isize) as *mut u32;
        *res_ptr = guest_ptr_offset as u32;
    }

    Ok(())
}

#[test]
fn test_allowed_domains() {
    let allowed_domains = vec![
        "https://api.brigade.sh".to_string(),
        "https://example.com".to_string(),
        "http://192.168.0.1".to_string(),
    ];

    assert_eq!(
        true,
        is_allowed(
            "https://api.brigade.sh/healthz",
            Some(allowed_domains.as_ref())
        )
        .unwrap()
    );
    assert_eq!(
        true,
        is_allowed(
            "https://example.com/some/path/with/more/paths",
            Some(allowed_domains.as_ref())
        )
        .unwrap()
    );
    assert_eq!(
        true,
        is_allowed("http://192.168.0.1/login", Some(allowed_domains.as_ref())).unwrap()
    );
    assert_eq!(
        false,
        is_allowed("https://test.brigade.sh", Some(allowed_domains.as_ref())).unwrap()
    );
}
