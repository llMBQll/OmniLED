use common_rs::interface::{Context, StatusCode, ManagedString};

extern {
    fn initialize_impl(ctx: *mut Context) -> StatusCode;
    fn display_name_impl(ctx: Context, str: *mut ManagedString) -> StatusCode;
    fn types_impl(ctx: Context, str: *mut ManagedString) -> StatusCode;
    fn update_impl(ctx: Context, str: *mut ManagedString) -> StatusCode;
    fn finalize_impl(ctx: Context) -> StatusCode;
}

#[no_mangle]
pub unsafe extern "C" fn initialize(ctx: *mut Context) -> StatusCode {
    initialize_impl(ctx)
}

#[no_mangle]
pub unsafe extern "C" fn display_name(ctx: Context, str: *mut ManagedString) -> StatusCode {
    display_name_impl(ctx, str)
}

#[no_mangle]
pub unsafe extern "C" fn types(ctx: Context, str: *mut ManagedString) -> StatusCode {
    types_impl(ctx, str)
}

#[no_mangle]
pub unsafe extern "C" fn update(ctx: Context, str: *mut ManagedString) -> StatusCode {
    update_impl(ctx, str)
}

#[no_mangle]
pub unsafe extern "C" fn finalize(ctx: Context) -> StatusCode {
    finalize_impl(ctx)
}