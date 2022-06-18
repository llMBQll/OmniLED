use common_rs::interface::{Context, StatusCode, ManagedString};

extern {
    fn initialize_impl(ctx: *mut *mut Context) -> StatusCode;
    fn name_impl(ctx: *mut Context, str: *mut ManagedString) -> StatusCode;
    fn types_impl(ctx: *mut Context, str: *mut ManagedString) -> StatusCode;
    fn update_impl(ctx: *mut Context, str: *mut ManagedString) -> StatusCode;
    fn finalize_impl(ctx: *mut Context) -> StatusCode;
}

#[no_mangle]
pub unsafe extern "C" fn initialize(ctx: *mut *mut Context) -> StatusCode {
    initialize_impl(ctx)
}

#[no_mangle]
pub unsafe extern "C" fn name(ctx: *mut Context, str: *mut ManagedString) -> StatusCode {
    name_impl(ctx, str)
}

#[no_mangle]
pub unsafe extern "C" fn types(ctx: *mut Context, str: *mut ManagedString) -> StatusCode {
    types_impl(ctx, str)
}

#[no_mangle]
pub unsafe extern "C" fn update(ctx: *mut Context, str: *mut ManagedString) -> StatusCode {
    update_impl(ctx, str)
}

#[no_mangle]
pub unsafe extern "C" fn finalize(ctx: *mut Context) -> StatusCode {
    finalize_impl(ctx)
}