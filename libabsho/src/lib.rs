use runtime::prepare::{self, ModelInfo, ShippingInfo, PaymentInfo, CookieData, UserData};
use runtime::task_ng::{SelectedGet, SelectedPlaceOrder, ChannelItemOptionInfo};
use runtime::task::{self};
use runtime::task_ng::{self};
use runtime::voucher::{self};
use runtime::crypt::{self};
use runtime::telegram::{self};
use std::os::raw::c_char;
use std::ffi::CString;
use std::ffi::CStr;

#[repr(C)]
pub struct AccountInfo {
    username: *mut c_char,
    email: *mut c_char,
    phone: *mut c_char,
    error: *mut c_char,
}
#[repr(C)]
pub struct FFICookieData {
    cookie_content: *mut c_char,
    csrftoken: *mut c_char,
}
#[repr(C)]
pub struct FFIAddressInfo  {
    state: *mut c_char,
    city: *mut c_char,
    district: *mut c_char,
    id: i64,
    error: *mut c_char,
}

#[unsafe(no_mangle)]
pub extern "C" fn create_cookie(cookie_content: *const c_char) -> FFICookieData {
    // Handle null pointer input
    if cookie_content.is_null() {
        return FFICookieData {
            cookie_content: std::ptr::null_mut(),
            csrftoken: std::ptr::null_mut(),
        };
    }

    // Convert C string to Rust &str dengan pengecekan error
    let cookie_content_str = unsafe { CStr::from_ptr(cookie_content) };
    let cookie_content_str = match cookie_content_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            return FFICookieData {
                cookie_content: std::ptr::null_mut(),
                csrftoken: std::ptr::null_mut(),
            };
        }
    };

    // Panggil fungsi internal
    let cookie_data = prepare::create_cookie(cookie_content_str);

    // Konversi dari PrepareCookieData ke FFICookieData
    let cookie_content_cstr = match CString::new(cookie_data.cookie_content) {
        Ok(c) => c,
        Err(_) => {
            return FFICookieData {
                cookie_content: std::ptr::null_mut(),
                csrftoken: std::ptr::null_mut(),
            };
        }
    };

    let csrftoken_cstr = match CString::new(cookie_data.csrftoken) {
        Ok(c) => c,
        Err(_) => {
            return FFICookieData {
                cookie_content: std::ptr::null_mut(),
                csrftoken: std::ptr::null_mut(),
            };
        }
    };

    FFICookieData {
        cookie_content: cookie_content_cstr.into_raw(),
        csrftoken: csrftoken_cstr.into_raw(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn address(cookie_content: &FFICookieData) -> FFIAddressInfo {
    let cookie_content = unsafe {
        CookieData {
            cookie_content: CStr::from_ptr(cookie_content.cookie_content)
                .to_string_lossy()
                .into_owned(),
            csrftoken: CStr::from_ptr(cookie_content.csrftoken)
                .to_string_lossy()
                .into_owned(),
        }
    };
    let result = tokio::runtime::Runtime::new().and_then(|rt| {
        Ok(rt.block_on(async {
            prepare::address(&cookie_content).await
        }))
    });

    match result {
        Ok(Ok(data)) => FFIAddressInfo  {
            state: CString::new(data.state).unwrap().into_raw(),
            city: CString::new(data.city).unwrap().into_raw(),
            district: CString::new(data.district).unwrap().into_raw(),
            id: data.id,
            error: std::ptr::null_mut(),
        },
        Ok(Err(e)) => FFIAddressInfo  {
            state: std::ptr::null_mut(),
            city: std::ptr::null_mut(),
            district: std::ptr::null_mut(),
            id: 0,
            error: CString::new(e.to_string()).unwrap().into_raw(),
        },
        Err(e) => FFIAddressInfo  {
            state: std::ptr::null_mut(),
            city: std::ptr::null_mut(),
            district: std::ptr::null_mut(),
            id : 0,
            error: CString::new(e.to_string()).unwrap().into_raw(),
        },
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn info_akun(cookie_content: &FFICookieData) -> AccountInfo {
    let cookie_content = unsafe {
        CookieData {
            cookie_content: CStr::from_ptr(cookie_content.cookie_content)
                .to_string_lossy()
                .into_owned(),
            csrftoken: CStr::from_ptr(cookie_content.csrftoken)
                .to_string_lossy()
                .into_owned(),
        }
    };
    let result = tokio::runtime::Runtime::new().and_then(|rt| {
        Ok(rt.block_on(async {
            prepare::info_akun(&cookie_content).await
        }))
    });

    match result {
        Ok(Ok(user_info)) => AccountInfo {
            username: CString::new(user_info.username).unwrap().into_raw(),
            email: CString::new(user_info.email).unwrap().into_raw(),
            phone: CString::new(user_info.phone).unwrap().into_raw(),
            error: std::ptr::null_mut(),
        },
        Ok(Err(e)) => AccountInfo {
            username: std::ptr::null_mut(),
            email: std::ptr::null_mut(),
            phone: std::ptr::null_mut(),
            error: CString::new(e.to_string()).unwrap().into_raw(),
        },
        Err(e) => AccountInfo {
            username: std::ptr::null_mut(),
            email: std::ptr::null_mut(),
            phone: std::ptr::null_mut(),
            error: CString::new(e.to_string()).unwrap().into_raw(),
        },
    }
}
