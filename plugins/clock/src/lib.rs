use std::ffi::c_void;

type Context = *mut c_void;

struct Data {
    tmp: String,
}

impl Data {
    pub fn new(tmp: String) -> Self {
        Self { tmp }
    }
}

#[no_mangle]
pub unsafe extern "C" fn initialize(ctx: *mut Context) -> i32 {
    let data = Box::new(Data::new(String::from("Hello from plugin")));
    let data_ptr: *mut Data = Box::into_raw(data);
    *(ctx) = data_ptr as *mut c_void;
    0
}

#[no_mangle]
pub unsafe extern "C" fn update(ctx: Context) -> i32 {
    let data = ctx as *const Data;
    println!("{}", (*data).tmp);
    0
}

#[no_mangle]
pub unsafe extern "C" fn finalize(ctx: Context) -> i32 {
    let _ = Box::from_raw(ctx as *mut Data);
    0
}