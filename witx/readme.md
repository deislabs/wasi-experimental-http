
# Module: wasi_experimental_http

## Table of contents

### Types list:

[**[All](#types)**] - [_[`http_error`](#http_error)_] - [_[`status_code`](#status_code)_] - [_[`outgoing_body`](#outgoing_body)_] - [_[`incoming_body`](#incoming_body)_] - [_[`response_handle`](#response_handle)_] - [_[`header_value_buf`](#header_value_buf)_] - [_[`written_bytes`](#written_bytes)_]

### Functions list:

[**[All](#functions)**] - [[`req()`](#req)] - [[`close()`](#close)] - [[`header_get()`](#header_get)] - [[`headers_get_all()`](#headers_get_all)] - [[`body_read()`](#body_read)]

## Types

### _[`http_error`](#http_error)_

Enumeration with tag type: `u32`, and the following members:

* **`success`**: _[`http_error`](#http_error)_
* **`invalid_handle`**: _[`http_error`](#http_error)_
* **`memory_not_found`**: _[`http_error`](#http_error)_
* **`memory_access_error`**: _[`http_error`](#http_error)_
* **`buffer_too_small`**: _[`http_error`](#http_error)_
* **`header_not_found`**: _[`http_error`](#http_error)_
* **`utf8_error`**: _[`http_error`](#http_error)_
* **`destination_not_allowed`**: _[`http_error`](#http_error)_
* **`invalid_method`**: _[`http_error`](#http_error)_
* **`invalid_encoding`**: _[`http_error`](#http_error)_
* **`invalid_url`**: _[`http_error`](#http_error)_
* **`request_error`**: _[`http_error`](#http_error)_
* **`runtime_error`**: _[`http_error`](#http_error)_
* **`too_many_sessions`**: _[`http_error`](#http_error)_

---

### _[`status_code`](#status_code)_
Alias for `u16`.


> HTTP status code


---

### _[`outgoing_body`](#outgoing_body)_
Alias for `u8` slice.


> An HTTP body being sent


---

### _[`incoming_body`](#incoming_body)_
Alias for `u8` mutable slice.


> Buffer for an HTTP body being received


---

### _[`response_handle`](#response_handle)_
Alias for `handle`.


> A response handle


---

### _[`header_value_buf`](#header_value_buf)_
Alias for `u8` mutable slice.


> Buffer to store a header value


---

### _[`written_bytes`](#written_bytes)_
Alias for `usize`.


> Number of bytes having been written


---

## Functions

### [`req()`](#req)
Returned error type: _[`http_error`](#http_error)_

#### Input:

* **`url`**: `string`
* **`method`**: `string`
* **`headers`**: `string`
* **`body`**: _[`outgoing_body`](#outgoing_body)_

#### Output:

* _[`status_code`](#status_code)_ mutable pointer
* _[`response_handle`](#response_handle)_ mutable pointer

> Send a request


---

### [`close()`](#close)
Returned error type: _[`http_error`](#http_error)_

#### Input:

* **`response_handle`**: _[`response_handle`](#response_handle)_

This function has no output.

> Close a request handle


---

### [`header_get()`](#header_get)
Returned error type: _[`http_error`](#http_error)_

#### Input:

* **`response_handle`**: _[`response_handle`](#response_handle)_
* **`header_name`**: `string`
* **`header_value_buf`**: _[`header_value_buf`](#header_value_buf)_

#### Output:

* _[`written_bytes`](#written_bytes)_ mutable pointer

> Get the value associated with a header


---

### [`headers_get_all()`](#headers_get_all)
Returned error type: _[`http_error`](#http_error)_

#### Input:

* **`response_handle`**: _[`response_handle`](#response_handle)_
* **`header_value_buf`**: _[`header_value_buf`](#header_value_buf)_

#### Output:

* _[`written_bytes`](#written_bytes)_ mutable pointer

> Get the entire response header map


---

### [`body_read()`](#body_read)
Returned error type: _[`http_error`](#http_error)_

#### Input:

* **`response_handle`**: _[`response_handle`](#response_handle)_
* **`body_buf`**: _[`incoming_body`](#incoming_body)_

#### Output:

* _[`written_bytes`](#written_bytes)_ mutable pointer

> Fill a buffer with the streamed content of a response body


---

