use common_rs::interface::{Context, ManagedString, StatusCode};
use crate::audio::Audio;

mod audio;
mod winapi;

#[no_mangle]
pub unsafe extern "C" fn initialize(ctx: *mut *mut Context) -> StatusCode {
    if winapi::initialize().is_err() {
        return StatusCode::Error
    }

    let audio = match Audio::new() {
        Ok(volume) => volume,
        Err(_) => return StatusCode::Error
    };

    let audio = Box::new(audio);
    let audio_ptr: *mut Audio = Box::into_raw(audio);
    *(ctx) = audio_ptr as *mut Context;

    StatusCode::Ok
}

#[no_mangle]
pub unsafe extern "C" fn name(_ctx: *mut Context, str: *mut ManagedString) -> StatusCode {
    *str = ManagedString::from(&String::from("AUDIO"));
    StatusCode::Ok
}

#[no_mangle]
pub unsafe extern "C" fn types(_ctx: *mut Context, str: *mut ManagedString) -> StatusCode {
    *str = ManagedString::from(&String::from(
        r#"{"Volume": "number", "IsMuted": "bool", "Name": "string"}"#
    ));
    StatusCode::Ok
}

#[no_mangle]
pub unsafe extern "C" fn update(ctx: *mut Context, str: *mut ManagedString) -> StatusCode {
    let audio = ctx as *mut Audio;
    match (*audio).update() {
        Ok(updated) => {
            if updated {
                *str = ManagedString::from(&String::from(
                    format!(r#"{{"Volume":{},"IsMuted":{},"Name":"{}"}}"#, (*audio).volume, (*audio).is_muted, (*audio).name)
                ));
            }
            StatusCode::Ok
        },
        Err(_) => StatusCode::Error
    }
}

#[no_mangle]
pub unsafe extern "C" fn finalize(ctx: *mut Context) -> StatusCode {
    let _ = Box::from_raw(ctx as *mut Audio);
    winapi::finalize();

    StatusCode::Ok
}