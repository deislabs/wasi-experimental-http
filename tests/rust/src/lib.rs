use bytes::Bytes;
use http;
use wasi_experimental_http;

#[no_mangle]
pub extern "C" fn get() {
    let url = "https://api.brigade.sh/healthz".to_string();
    let req = http::request::Builder::new().uri(&url).body(None).unwrap();
    let res = wasi_experimental_http::request(req).expect("cannot make get request");

    let str = std::str::from_utf8(&res.body()).unwrap().to_string();
    assert_eq!(r#""OK""#, str);
    // println!("{}", str);
}

#[no_mangle]
pub extern "C" fn post() {
    let url = "https://postman-echo.com/post".to_string();
    let req = http::request::Builder::new()
        .method(http::Method::POST)
        .uri(&url)
        .header("Content-Type", "text/plain")
        .header("abc", "def");
    let b = Bytes::from("Testing with a request body. Does this actually work?");
    let req = req.body(Some(b)).unwrap();

    let res = wasi_experimental_http::request(req).expect("cannot make post request");
    let _ = std::str::from_utf8(&res.body()).unwrap().to_string();
    // println!("{:#?}", res.headers());
    // println!("{}", str);
    // println!("{:#?}", res.status().to_string());
}
