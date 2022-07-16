// use serde_json::json;
// use common_rs::interface::{Context, ManagedString, StatusCode};
// use crate::spotify::Spotify;
//
// mod spotify;
//
// #[no_mangle]
// pub unsafe extern "C" fn initialize(ctx: *mut *mut Context) -> StatusCode {
//     let spotify = Box::new(Spotify::new());
//     let spotify_ptr: *mut Spotify = Box::into_raw(spotify);
//     *(ctx) = spotify_ptr as *mut Context;
//
//     StatusCode::Ok
// }
//
// #[no_mangle]
// pub unsafe extern "C" fn name(_ctx: *mut Context, str: *mut ManagedString) -> StatusCode {
//     *str = ManagedString::from(&String::from("SPOTIFY"));
//     StatusCode::Ok
// }
//
// #[no_mangle]
// pub unsafe extern "C" fn types(_ctx: *mut Context, str: *mut ManagedString) -> StatusCode {
//     *str = ManagedString::from(&String::from(
//         // r#"{"Artist": "string", "Track": "string", "Duration": "number", "Progress": "number"}"#
//         r#" {"Artist": "string", "Track": "string"} "#
//     ));
//     StatusCode::Ok
// }
//
// #[no_mangle]
// pub unsafe extern "C" fn update(ctx: *mut Context, str: *mut ManagedString) -> StatusCode {
//     let spotify = ctx as *mut Spotify;
//     match (*spotify).update() {
//         Some((artist, track)) => {
//             *str = ManagedString::from(
//                 &json!({
//                     "Artist": artist,
//                     "Track": track
//                 }).to_string()
//             );
//         }
//         None => {}
//     }
//     StatusCode::Ok
// }
//
// #[no_mangle]
// pub unsafe extern "C" fn finalize(ctx: *mut Context) -> StatusCode {
//     let _ = Box::from_raw(ctx as *mut Spotify);
//
//     StatusCode::Ok
// }