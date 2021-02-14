use anyhow::Error;
use futures::executor::block_on;
use reqwest::{Client, Method};
use wasmtime::*;

const ALLOC_FN: &str = "alloc";
const MEMORY: &str = "memory";

pub fn link_http(linker: &mut Linker) -> Result<(), Error> {
    linker.func(
        "wasi_experimental_http",
        "req",
        move |caller: Caller<'_>, url_ptr: u32, url_len_ptr: u32, written_ptr: u32| -> u32 {
            let memory = match caller.get_export(MEMORY) {
                Some(Extern::Memory(mem)) => mem,
                _ => panic!("cannot find memory"),
            };

            let url = unsafe { string_from_memory(&memory, url_ptr, url_len_ptr).unwrap() };
            println!("wasi_experimental_http::req: URL: {}", url);

            // TODO
            // We probably need separate methods for blocking and non-blocking
            // versions of the HTTP client.
            // let res = reqwest::blocking::get(&url).unwrap().text().unwrap();

            let client = Client::builder().build().unwrap();
            let res = block_on(client.request(Method::GET, &url).send()).unwrap();
            let res = block_on(res.text()).unwrap();

            println!("wasi_experimental_http: response: {}", res);

            write_guest_memory(&res.as_bytes().to_vec(), written_ptr, caller).unwrap() as u32
        },
    )?;

    Ok(())
}

/// Read a byte array from the instance's `memory`  of length `len_ptr`
/// starting at offset `data_ptr`
unsafe fn data_from_memory(memory: &Memory, data_ptr: u32, len_ptr: u32) -> (Option<&[u8]>, u32) {
    let len_ptr = memory.data_ptr().offset(len_ptr as isize) as *const u32;
    let len = *len_ptr;

    println!("wasi_experimental_http::data_from_memory:: length: {}", len);

    let data = memory
        .data_unchecked()
        .get(data_ptr as u32 as usize..)
        .and_then(|arr| arr.get(..len as u32 as usize));

    return (data, len);
}

/// Read a string from the instance's `memory`  of length `len_ptr`
/// starting at offset `data_ptr`
unsafe fn string_from_memory(
    memory: &Memory,
    data_ptr: u32,
    len_ptr: u32,
) -> Result<String, anyhow::Error> {
    let (data, _) = data_from_memory(&memory, data_ptr, len_ptr);
    let str = match data {
        Some(data) => match std::str::from_utf8(data) {
            Ok(s) => s,
            Err(_) => return Err(anyhow::Error::msg("invalid utf-8")),
        },
        None => return Err(anyhow::Error::msg("pointer/length out of bounds")),
    };

    // println!("wasi_experimental_http::string_from_memory:: data: {}", str);

    Ok(String::from(str))
}

/// Write a bytes array into the instance's linear memory
/// and return the offset relative to the module's memory.
fn write_guest_memory(bytes: &Vec<u8>, written: u32, caller: Caller<'_>) -> Result<isize, Error> {
    // Get the "memory" export of the module.
    // If the module does not export it, just panic,
    // since we are not going to be able to copy the model and image.
    let memory = match caller.get_export(MEMORY) {
        Some(Extern::Memory(mem)) => mem,
        _ => return Err(Error::msg("cannot find memory")),
    };

    // The module is not using any bindgen libraries, so it should export
    // its own alloc function.
    //
    // Get the guest's exported alloc function, and call it with the
    // length of the byte array we are trying to copy.
    // The result is an offset relative to the module's linear memory, which is
    // used to copy the bytes into the module's memory.
    // Then, return the offset.

    let alloc = match caller.get_export(ALLOC_FN) {
        Some(Extern::Func(func)) => func,
        _ => return Err(Error::msg("cannot find alloc function")),
    };
    let alloc_result = alloc.call(&vec![Val::from(bytes.len() as i32)])?;

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
        let written_ptr = memory.data_ptr().offset(written as isize) as *mut u32;
        *written_ptr = bytes.len() as u32;
        println!(
            "wasi_experimental_http::write_guest_memory:: written {} bytes",
            *written_ptr
        );
    }

    Ok(guest_ptr_offset)
}
