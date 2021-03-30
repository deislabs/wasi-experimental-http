use bytes::Bytes;

#[no_mangle]
pub extern "C" fn get() {
    let url = "https://api.brigade.sh/healthz".to_string();
    let req = http::request::Builder::new().uri(&url).body(None).unwrap();
    let mut res = wasi_experimental_http::request(req).expect("cannot make get request");
    let str = std::str::from_utf8(&res.body_read_all().unwrap())
        .unwrap()
        .to_string();
    assert_eq!(str, r#""OK""#);
    assert_eq!(res.status_code, 200);
    assert!(!res.header_get("content-type").unwrap().is_empty());
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

    let mut res = wasi_experimental_http::request(req).expect("cannot make post request");
    let _ = std::str::from_utf8(&res.body_read_all().unwrap())
        .unwrap()
        .to_string();
    assert_eq!(res.status_code, 200);
    assert!(!res.header_get("content-type").unwrap().is_empty());
}
