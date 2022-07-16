#[repr(i32)]
#[derive(Debug)]
pub enum StatusCode {
    Ok = 0,
    Error = 1,
}

pub type OnUpdateCallbackFn = extern fn(*const u8, u32) -> StatusCode;
pub type RunFn = extern fn(*const i32, OnUpdateCallbackFn) -> StatusCode;