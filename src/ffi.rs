use crate::{
    extract_token, fetch_cookies, fetch_token, init_logging, login, logout, CookieJar, CONFIG,
};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

pub struct CookieJarHandle(CookieJar);

#[no_mangle]
pub extern "C" fn libauthcekunit_init_logging() {
    init_logging();
}

#[no_mangle]
pub unsafe extern "C" fn libauthcekunit_login(base_url: *const c_char) -> *mut CookieJarHandle {
    if base_url.is_null() {
        return ptr::null_mut();
    }
    let url = unsafe { CStr::from_ptr(base_url).to_str().unwrap_or("") };
    match login(url) {
        Ok(jar) => Box::into_raw(Box::new(CookieJarHandle(jar))),
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn libauthcekunit_logout(
    handle: *mut CookieJarHandle,
    base_url: *const c_char,
) -> i32 {
    if handle.is_null() {
        return -1;
    }
    let jar = unsafe { Box::from_raw(handle) };
    let url = if base_url.is_null() {
        ""
    } else {
        unsafe { CStr::from_ptr(base_url).to_str().unwrap_or("") }
    };
    match logout(url, &jar.0) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn libauthcekunit_free_jar(handle: *mut CookieJarHandle) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn libauthcekunit_fetch_token(
    url: *const c_char,
    out: *mut c_char,
    out_size: usize,
) -> usize {
    if url.is_null() || out.is_null() || out_size == 0 {
        return 0;
    }
    let url_str = unsafe { CStr::from_ptr(url).to_str().unwrap_or("") };
    match fetch_token(url_str) {
        Ok(token) => {
            let c_token = match CString::new(token) {
                Ok(c) => c,
                Err(_) => return 0,
            };
            let bytes = c_token.as_bytes_with_nul();
            if bytes.len() > out_size {
                return 0;
            }
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), out as *mut u8, bytes.len());
            }
            bytes.len()
        }
        Err(_) => 0,
    }
}

#[no_mangle]
pub unsafe extern "C" fn libauthcekunit_fetch_cookies(
    url: *const c_char,
    file_path: *const c_char,
) -> i32 {
    if url.is_null() || file_path.is_null() {
        return -1;
    }
    let url_str = unsafe { CStr::from_ptr(url).to_str().unwrap_or("") };
    let path_str = unsafe { CStr::from_ptr(file_path).to_str().unwrap_or("") };
    match fetch_cookies(url_str) {
        Ok(jar) => match jar.save_to_file(path_str) {
            Ok(_) => 0,
            Err(_) => -1,
        },
        Err(_) => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn libauthcekunit_extract_token(
    html: *const c_char,
    out: *mut c_char,
    out_size: usize,
) -> usize {
    if html.is_null() || out.is_null() || out_size == 0 {
        return 0;
    }
    let html_str = unsafe { CStr::from_ptr(html).to_str().unwrap_or("") };
    match extract_token(html_str) {
        Ok(token) => {
            let c_token = match CString::new(token) {
                Ok(c) => c,
                Err(_) => return 0,
            };
            let bytes = c_token.as_bytes_with_nul();
            if bytes.len() > out_size {
                return 0;
            }
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), out as *mut u8, bytes.len());
            }
            bytes.len()
        }
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn libauthcekunit_get_email() -> *const c_char {
    CONFIG.email.as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn libauthcekunit_get_user_agent() -> *const c_char {
    CONFIG.user_agent.as_ptr() as *const c_char
}
