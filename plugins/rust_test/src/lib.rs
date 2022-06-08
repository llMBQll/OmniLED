use std::ffi::c_void;
use common_rs::managed_string::ManagedString;

type Context = *mut c_void;

struct Data {
    msg: String
}

impl Data {
    pub fn new() -> Self {
        Self {
            msg: String::from("Hello from Rust plugin")
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn initialize(ctx: *mut Context) -> i32 {
    let data = Box::new(Data::new());
    let data_ptr: *mut Data = Box::into_raw(data);
    *(ctx) = data_ptr as *mut c_void;
    0
}

#[no_mangle]
pub unsafe extern "C" fn display_name(ctx: Context, str: *mut ManagedString) -> i32 {
    let data = ctx as *const Data;
    *str = ManagedString::from(&(*data).msg);
    0
}

#[no_mangle]
pub unsafe extern "C" fn types(ctx: Context, str: *mut ManagedString) -> i32 {
    let data = ctx as *const Data;
    *str = ManagedString::from(&(*data).msg);
    0
}

#[no_mangle]
pub unsafe extern "C" fn update(ctx: Context, str: *mut ManagedString) -> i32 {
    let data = ctx as *const Data;
    *str = ManagedString::from(&(*data).msg);
    0
}

#[no_mangle]
pub unsafe extern "C" fn finalize(ctx: Context) -> i32 {
    let _ = Box::from_raw(ctx as *mut Data);
    0
}