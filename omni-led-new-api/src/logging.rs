use log::{Log, Metadata, Record, error};

use crate::rust_api::OmniLedApi;

pub fn init(api: OmniLedApi, crate_name: &'static str) {
    let logger = Logger::new(api, crate_name);
    log::set_boxed_logger(Box::new(logger))
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        error!("{panic_info}");
        default_hook(panic_info);
    }));
}

struct Logger {
    api: OmniLedApi,
    crate_name: &'static str,
}

impl Logger {
    pub fn new(api: OmniLedApi, crate_name: &'static str) -> Self {
        Self { api, crate_name }
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let target = metadata.target();
        target.starts_with(self.crate_name) || target.starts_with("omni_led")
    }

    fn log(&self, record: &Record) {
        if !self.enabled(&record.metadata()) {
            return;
        }

        let log_level = record.level();
        let target = record.target();
        let message = record.args().to_string();

        self.api.log(log_level, target, &message);
    }

    fn flush(&self) {}
}
