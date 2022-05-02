use bytes::Bytes;

#[no_mangle]
pub extern "C" fn get() {
    let url = "https://some-random-api.ml/facts/dog".to_string();
    let req = http::request::Builder::new().uri(&url).body(None).unwrap();
    let mut res = wasi_experimental_http::request(req).expect("cannot make get request");
    let str = std::str::from_utf8(&res.body_read_all().unwrap())
        .unwrap()
        .to_string();
    assert_eq!(str.is_empty(), false);
    assert_eq!(res.status_code, 200);
    assert!(!res
        .header_get("content-type".to_string())
        .unwrap()
        .is_empty());

    let header_map = res.headers_get_all().unwrap();
    assert_ne!(header_map.len(), 0);
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
    assert!(!res
        .header_get("content-type".to_string())
        .unwrap()
        .is_empty());

    let header_map = res.headers_get_all().unwrap();
    assert_ne!(header_map.len(), 0);
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn concurrent() {
    let url = "https://some-random-api.ml/facts/dog".to_string();
    // the responses are unused to avoid dropping them.
    let req1 = make_req(url.clone());
    let req2 = make_req(url.clone());
    let req3 = make_req(url);
}

fn make_req(url: String) -> wasi_experimental_http::Response {
    let req = http::request::Builder::new().uri(&url).body(None).unwrap();
    wasi_experimental_http::request(req).expect("cannot make get request")
}
