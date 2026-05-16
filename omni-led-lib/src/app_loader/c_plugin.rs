use std::{io::Cursor, slice};

use omni_led_api::{c_api, types::EventData};
use prost::Message;

use crate::{
    app_loader::process::Config,
    events::event_queue::{Event, EventQueue},
};

pub struct CPlugin;

impl CPlugin {
    pub fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        let config = config.clone();

        // TODO join it later
        std::thread::spawn(move || unsafe {
            let lib = libloading::Library::new(&config.path).unwrap();
            let omni_led_run_fn: libloading::Symbol<<c_api::omni_led_run_t as FnPtr>::Type> =
                lib.get(b"omni_led_run").unwrap();

            // TODO pass args
            (omni_led_run_fn)(
                c_api::OmniLedApi {
                    event: Some(plugin_event),
                    log: Some(plugin_log),
                },
                0,
                std::ptr::null_mut(),
            );
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

    let message = unsafe { slice::from_raw_parts(message as *const u8, message_length as usize) };
    // TODO error handling
    let message = str::from_utf8(message).unwrap();

    log::log!(target: target, level, "{}", message);
}
