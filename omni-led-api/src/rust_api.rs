use log::Level;

use crate::c_api;

#[derive(Clone, Copy)]
pub struct OmniLedApi {
    event_fn: <c_api::omni_led_event_t as FnPtr>::Type,
    log_fn: <c_api::omni_led_log_t as FnPtr>::Type,
}

impl OmniLedApi {
    pub fn new(c_api: c_api::OmniLedApi) -> Self {
        Self {
            event_fn: c_api.event.unwrap(),
            log_fn: c_api.log.unwrap(),
        }
    }

    pub fn event(&self, event_data: &[u8]) {
        unsafe { (self.event_fn)(event_data.as_ptr(), event_data.len() as u64) }
    }

    pub fn log(&self, log_level: Level, target: &str, message: &str) {
        let log_level = match log_level {
            Level::Error => c_api::LogLevel_LOG_LEVEL_ERROR,
            Level::Warn => c_api::LogLevel_LOG_LEVEL_WARN,
            Level::Info => c_api::LogLevel_LOG_LEVEL_INFO,
            Level::Debug => c_api::LogLevel_LOG_LEVEL_DEBUG,
            Level::Trace => c_api::LogLevel_LOG_LEVEL_TRACE,
        };

        unsafe {
            (self.log_fn)(
                log_level.into(),
                target.as_ptr() as *const i8,
                target.len() as u64,
                message.as_ptr() as *const i8,
                message.len() as u64,
            )
        }
    }
}

trait FnPtr {
    type Type;
}

impl<T> FnPtr for Option<T> {
    type Type = T;
}
