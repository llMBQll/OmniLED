use std::{thread, time};
use common_rs::interface::{OnUpdateCallbackFn, StatusCode};

use crate::audio::Audio;

mod audio;
mod winapi;

const SLEEP_DURATION: time::Duration = time::Duration::from_millis(50);

#[no_mangle]
pub unsafe extern "C" fn run(keep_running: *const i32, on_update: OnUpdateCallbackFn) -> StatusCode {
    if winapi::initialize().is_err() {
        return StatusCode::Error;
    }

    let _audio = match Audio::new(on_update) {
        Ok(audio) => audio,
        Err(_) => return StatusCode::Error
    };

    while *keep_running == 1 {
        thread::sleep(SLEEP_DURATION);
    }

    winapi::finalize();
    StatusCode::Ok
}