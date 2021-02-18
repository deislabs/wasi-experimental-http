use http;
use wasi_http;

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let url = "https://api.brigade.sh/healthz".to_string();
    let req = http::request::Builder::new().uri(&url).body(()).unwrap();
    let res = wasi_http::request(req).unwrap();
    let str = std::str::from_utf8(&res.body()).unwrap().to_string();
    println!("{}", str);
}
