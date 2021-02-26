use http;
use wasi_experimental_http;

#[no_mangle]
pub extern "C" fn _start() {
    let url = "https://postman-echo.com/post".to_string();
    let req = http::request::Builder::new()
        .method(http::Method::POST)
        .uri(&url)
        .header("Content-Type", "text/plain");
    let b = b"Testing with a request body";
    let req = req.body(Some(b.to_vec())).unwrap();

    // let url = "https://api.brigade.sh/healthz".to_string();
    // let req = http::request::Builder::new().uri(&url).body(None).unwrap();

    let res = wasi_experimental_http::request(req).expect("cannot make request");
    let str = std::str::from_utf8(&res.body()).unwrap().to_string();
    println!("{:#?}", res.headers());
    println!("{}", str);
    println!("{:#?}", res.status().to_string());
}
