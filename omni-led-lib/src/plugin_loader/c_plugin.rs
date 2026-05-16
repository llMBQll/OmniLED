use std::ffi::{CString, c_char, c_int};
use std::io::Cursor;
use std::slice;
use std::str::FromStr;

use log::debug;
use mlua::UserData;
use omni_led_api::{c_api, types::EventData};
use omni_led_derive::FromLuaValue;
use prost::Message;

use crate::events::event_queue::{Event, EventQueue};

pub struct CPlugin;

impl CPlugin {
    pub fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        let config = config.clone();

        // TODO join it later
        std::thread::spawn(move || unsafe {
            let lib = libloading::Library::new(&config.path).unwrap();
            let omni_led_run_fn: libloading::Symbol<<c_api::omni_led_run_t as FnPtr>::Type> =
                lib.get(b"omni_led_run").unwrap();

            let mut args = config.args.clone();
            args.insert(0, config.path.clone());

            let args = args
                .iter()
                .map(|arg| CString::from_str(&arg).unwrap())
                .collect::<Vec<_>>();

            let ptr_args = args
                .iter()
                .map(|arg| arg.as_ptr() as *mut c_char)
                .collect::<Vec<_>>();

            let argc = args.len() as c_int;
            let argv = ptr_args.as_ptr() as *mut *mut c_char;

            let result = (omni_led_run_fn)(
                c_api::OmniLedApi {
                    event: Some(plugin_event),
                    log: Some(plugin_log),
                },
                argc,
                argv,
            );
            debug!("{:?} finished with code {}", config, result);
        });

        Ok(Self)
    }
}

trait FnPtr {
    type Type;
}

impl<T> FnPtr for Option<T> {
    type Type = T;
}

unsafe extern "C" fn plugin_event(
    event_data: *const ::std::os::raw::c_uchar,
    event_data_length: ::std::os::raw::c_ulonglong,
) {
    let event_data =
        unsafe { slice::from_raw_parts(event_data as *const u8, event_data_length as usize) };

    // TODO error handling
    let event_data = EventData::decode(&mut Cursor::new(event_data)).unwrap();

    let name = event_data.name;
    let fields = event_data.fields.unwrap();

    EventQueue::instance()
        .lock()
        .unwrap()
        .push(Event::Application((name, fields)));
}

unsafe extern "C" fn plugin_log(
    level: c_api::LogLevel,
    target: *const ::std::os::raw::c_char,
    target_length: ::std::os::raw::c_ulonglong,
    message: *const ::std::os::raw::c_char,
    message_length: ::std::os::raw::c_ulonglong,
) {
    let level = match level {
        c_api::LogLevel_LOG_LEVEL_ERROR => log::Level::Error,
        c_api::LogLevel_LOG_LEVEL_WARN => log::Level::Warn,
        c_api::LogLevel_LOG_LEVEL_INFO => log::Level::Info,
        c_api::LogLevel_LOG_LEVEL_DEBUG => log::Level::Debug,
        c_api::LogLevel_LOG_LEVEL_TRACE => log::Level::Trace,
        _ => {
            // TODO error handling
            panic!("Unknown log level '{level}'");
        }
    };

    let target = unsafe { slice::from_raw_parts(target as *const u8, target_length as usize) };
    // TODO error handling
    let target = str::from_utf8(target).unwrap();
    let target = format!("plugin::{}", target);

    let message = unsafe { slice::from_raw_parts(message as *const u8, message_length as usize) };
    // TODO error handling
    let message = str::from_utf8(message).unwrap();

    log::log!(target: &target, level, "{}", message);
}

#[derive(Debug, Clone, FromLuaValue)]
pub struct Config {
    path: String,
    #[mlua(default)]
    args: Vec<String>,
}

impl UserData for Config {}
