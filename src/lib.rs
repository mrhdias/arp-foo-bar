//
// plugin test as a shared library
//

use std::ffi::{
    c_char,
    CStr,
    CString,
};
use std::collections::HashMap;
use serde::Serialize;
use hyper::HeaderMap;
use url::form_urlencoded;
use once_cell::sync::OnceCell;

static VERSION: &'static str = "0.1.0";
// Define a global `OnceCell` for `KEY`
static KEY: OnceCell<String> = OnceCell::new();

#[derive(Debug, Serialize)]
struct PluginRoute {
    path: &'static str,
    function: &'static str,
    method_router: &'static str,
    response_type: &'static str,
}

static ROUTES: &[PluginRoute] = &[
    PluginRoute {
        path: "/test-get",
        function: "test_get",
        method_router: "get",
        response_type: "html",
    },
    PluginRoute {
        path: "/test-post",
        function: "test_post",
        method_router: "post",
        response_type: "html",
    },
    PluginRoute {
        path: "/test-json",
        function: "test_json",
        method_router: "post",
        response_type: "json",
    },
    PluginRoute {
        path: "/about",
        function: "about",
        method_router: "get",
        response_type: "text",
    },
];

#[no_mangle]
pub extern "C" fn test_json(
    headers: *mut HeaderMap,
    body: *const c_char,
) -> *const c_char {

    if headers.is_null() || body.is_null() {
        // Handle the null pointer case
        return std::ptr::null_mut();
    }

    // Convert headers pointer to a reference
    let headers = unsafe { &*headers };

    match headers.get("content-type") {
        Some(value) => {
            if value.to_str().unwrap_or("").to_string() != "application/json" {
                panic!("Invalid content type: {:?}", value);
            }
        },
        None => panic!("No content type"),
    };

    // Convert body pointer to a Rust string
    let body_str = unsafe {
        CStr::from_ptr(body)
            .to_str()
            .unwrap_or("Invalid UTF-8 sequence") // Handle possible UTF-8 errors
    };
    println!("Body Str: {}", body_str);

    // Deserialize JSON string to HashMap<String, Vec<String>>
    let bag: HashMap<String, Vec<String>> = serde_json::from_str(body_str).unwrap();

    // Printing the HashMap to verify
    for (key, value) in &bag {
        println!("{}: {:?}", key, value);
    }

    #[derive(Serialize)]
    struct Totals {
        fruits: usize,
        vegetables: usize,
    }

    let totals = Totals {
        fruits: bag.get("fruits").unwrap_or(&Vec::new()).len(),
        vegetables: bag.get("vegetables").unwrap_or(&Vec::new()).len(),
    };

    // Convert the Totals struct to a JSON string
    let json_string = serde_json::to_string(&totals).unwrap();

    let c_response = CString::new(json_string).unwrap();
    c_response.into_raw()
}

#[no_mangle]
pub extern "C" fn test_post(
    headers: *mut HeaderMap,
    body: *const c_char,
) -> *const c_char {

    if headers.is_null() || body.is_null() {
        // Handle the null pointer case
        return std::ptr::null_mut();
    }

    // Convert headers pointer to a reference
    let headers = unsafe { &*headers };

    // Convert body pointer to a Rust string
    let body_str = unsafe {
        CStr::from_ptr(body)
            .to_str()
            .unwrap_or("Invalid UTF-8 sequence") // Handle possible UTF-8 errors
    };

    // Parse and decode the URL-encoded form data
    let decoded_body: Vec<(String, String)> = form_urlencoded::parse(body_str.as_bytes())
        .into_owned()
        .collect();

    // only for debug purposes
    println!("Headers: {:?}", headers);
    println!("Decoded Body: {:?}", decoded_body);

    let c_response = CString::new(format!("{} : {}", decoded_body[0].0, decoded_body[0].1)).unwrap();
    c_response.into_raw()
}

#[no_mangle]
pub extern "C" fn test_get(
    headers: *mut HeaderMap,
    body: *const c_char,
) -> *const c_char {

    if headers.is_null() || body.is_null() {
        // Handle the null pointer case
        return std::ptr::null_mut();
    }

    // Convert headers pointer to a reference
    let headers = unsafe { &*headers };

    // Convert body pointer to a Rust string
    let body_str = unsafe {
        CStr::from_ptr(body)
            .to_str()
            .unwrap_or("Invalid UTF-8 sequence") // Handle possible UTF-8 errors
    };

    // only for debug purposes
    println!("Headers: {:?}", headers);
    println!("Body: {:?}", body_str);

    let c_response = CString::new(r#"
<form method="post" action="/plugin/foo-bar/test-post">
<input type="text" name="my_text" value="Lorem ipsum dolor sit amet, consectetur adipiscing elit. Cras sagittis quam id libero ultrices imperdiet. Nullam odio risus, ultricies quis ornare ut, bibendum eget tortor.">
<input type="submit" value="Submit">
</form>"#).unwrap();

    c_response.into_raw()
}

#[no_mangle]
pub extern "C" fn routes() -> *const c_char {

    /*
    if KEY.get().is_none() {
        return CString::new(r#"[
{
    "path": "/register",
    "function": "register",
    "method_router": "get",
    "response_type": "html"
}
]"#)
            .unwrap()
            .into_raw();
    }
    */

    let json_routes = serde_json::to_string_pretty(ROUTES)
        .unwrap_or("[]".to_string());

    let c_response = CString::new(json_routes.as_str())
        .unwrap();
    c_response
        .into_raw()
}

#[no_mangle]
pub extern "C" fn about(
    _headers: *mut HeaderMap,
    _body: *const c_char,
) -> *const c_char {

    let info = format!(r#"Name: arp-foo-bar
Version: {}
authors = "Henrique Dias <mrhdias@gmail.com>"
Description: Shared library Example
License: MIT"#, VERSION);

    let c_response = CString::new(info).unwrap();
    c_response.into_raw()
}

#[no_mangle]
pub extern "C" fn key(s: *const c_char) -> *const c_char {
    if s.is_null() {
        // Handle the null pointer case
        return std::ptr::null_mut();
    }

    // Convert body pointer to a Rust string
    let key_str = unsafe {
        match CStr::from_ptr(s).to_str() {
            Ok(s) => s,
            Err(_) => return CString::new(r#"{"message": "Invalid UTF-8 sequence", "status": 1}"#)
                .unwrap()
                .into_raw(),
        }
    };

    let (error, status) = || -> (&str, i32) {
        if key_str.is_empty() {
            return ("Key cannot be empty", 1);
        }
        // get client key from server
        if key_str != "e80b5017098950fc58aad83c8c14978e" {
            return ("Invalid key", 1);
        }
        ("", 0)
    }();

    if status > 0 {
        let json_str = format!(r#"{{"message": "{}", "status": {}}}"#, error, status);
        return CString::new(json_str).unwrap().into_raw();
    }

    // Initialize the global `KEY` only once
    match KEY.set(key_str.to_string()) {
        Ok(_) => {
            let json_str = r#"{"message": "Success", "status": 0}"#;
            let c_response = CString::new(json_str).unwrap();
            c_response.into_raw()
        }
        Err(_) => {
            let json_str = r#"{"message": "Key already initialized", "status": 1}"#;
            let c_response = CString::new(json_str).unwrap();
            c_response.into_raw()
        }
    }
}

#[no_mangle]
pub extern "C" fn free(ptr: *mut c_char) {
    if ptr.is_null() { // Avoid dereferencing null pointers
        return;
    }

    // Convert the raw pointer back to a CString and drop it to free the memory
    unsafe {
        drop(CString::from_raw(ptr)); // Takes ownership of the memory and frees it when dropped
    }
}