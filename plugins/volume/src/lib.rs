use std::{thread, time};
use common_rs::interface::{OnUpdateCallbackFn, RunFn, StatusCode};

use crate::audio::Audio;

mod audio;
mod winapi;

const SLEEP_DURATION: time::Duration = time::Duration::from_millis(50);

// #[no_mangle]
// pub unsafe extern "C" fn update(ctx: *mut Context, str: *mut ManagedString) -> StatusCode {
//     let audio = ctx as *mut Audio;
//     match (*audio).update() {
//         Ok(updated) => {
//             if updated {
//                 *str = ManagedString::from(&String::from(
//                     format!(r#"{{"Volume":{},"IsMuted":{},"Name":"{}"}}"#, (*audio).volume, (*audio).is_muted, (*audio).name)
//                 ));
//             }
//             StatusCode::Ok
//         },
//         Err(_) => StatusCode::Error
//     }
// }


#[no_mangle]
pub unsafe extern "C" fn run(keep_running: *const i32, on_update: OnUpdateCallbackFn) -> StatusCode {
    if winapi::initialize().is_err() {
        return StatusCode::Error;
    }

    audio = Audio::new();

    while *keep_running == 1 {
        thread::sleep(SLEEP_DURATION);
    }

    winapi::finalize();
    StatusCode::Ok
}