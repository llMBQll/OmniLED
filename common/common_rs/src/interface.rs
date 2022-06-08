use std::ffi::c_void;

pub use crate::managed_string::ManagedString;

#[repr(i32)]
#[derive(Debug)]
pub enum StatusCode {
    Ok = 0,
    Error = 1,
}

pub type Context = *mut c_void;
pub type InitializeFn = fn(*mut Context) -> StatusCode;
pub type DisplayNameFn = fn(Context, *mut ManagedString) -> StatusCode;
pub type TypesFn = fn(Context, *mut ManagedString) -> StatusCode;
pub type UpdateFn = fn(Context, *mut ManagedString) -> StatusCode;
pub type FinalizeFn = fn(Context) -> StatusCode;