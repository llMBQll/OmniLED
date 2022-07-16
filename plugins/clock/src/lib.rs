use common_rs::interface::{OnUpdateCallbackFn, StatusCode};

extern {
    fn run_impl(keep_running: *const i32, on_update: OnUpdateCallbackFn) -> StatusCode;
}

#[no_mangle]
pub unsafe extern "C" fn run(keep_running: *const i32, on_update: OnUpdateCallbackFn) -> StatusCode {
    run_impl(keep_running, on_update)
}