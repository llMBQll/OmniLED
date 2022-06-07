use std::ffi::c_void;

use common_rs::managed_string::ManagedString;

type Context = *mut c_void;

extern {
    fn initialize_impl(ctx: *mut Context) -> i32;
    fn display_name_impl(ctx: Context, str: *mut ManagedString) -> i32;
    fn types_impl(ctx: Context, str: *mut ManagedString) -> i32;
    fn update_impl(ctx: Context, str: *mut ManagedString) -> i32;
    fn finalize_impl(ctx: Context) -> i32;
}

#[no_mangle]
pub unsafe extern "C" fn initialize(ctx: *mut Context) -> i32 {
    initialize_impl(ctx)
}

#[no_mangle]
pub unsafe extern "C" fn display_name(ctx: Context, str: *mut ManagedString) -> i32 {
    display_name_impl(ctx, str)
}

#[no_mangle]
pub unsafe extern "C" fn types(ctx: Context, str: *mut ManagedString) -> i32 {
    types_impl(ctx, str)
}

#[no_mangle]
pub unsafe extern "C" fn update(ctx: Context, str: *mut ManagedString) -> i32 {
    update_impl(ctx, str)
}

#[no_mangle]
pub unsafe extern "C" fn finalize(ctx: Context) -> i32 {
    finalize_impl(ctx)
}