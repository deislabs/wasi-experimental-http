use http;
use wasi_experimental_http;

#[no_mangle]
pub extern "C" fn _start() {
    let url = "https://api.brigade.sh/healthz".to_string();
    let req = http::request::Builder::new().uri(&url).header("abc", "def");
    let req = req.body(()).unwrap();

    let res = wasi_experimental_http::request(req).expect("cannot make request");
    let str = std::str::from_utf8(&res.body()).unwrap().to_string();
    println!("{:#?}", res.headers());
    println!("{}", str);
    println!("{:#?}", res.status());
}
