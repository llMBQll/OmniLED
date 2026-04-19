use log::{Log, Metadata, Record, error};
use tokio::runtime::Handle;

use crate::plugin::Plugin;

pub fn init(runtime_handle: Handle, plugin: Plugin, crate_name: &'static str) {
    let logger = Logger::new(runtime_handle, plugin, crate_name);
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
    runtime_handle: Handle,
    plugin: Plugin,
    crate_name: &'static str,
}

impl Logger {
    pub fn new(runtime_handle: Handle, plugin: Plugin, crate_name: &'static str) -> Self {
        Self {
            runtime_handle,
            plugin,
            crate_name,
        }
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
        let target = record.target().to_string();
        let message = format!("{}", record.args());
        let plugin = self.plugin.clone();
        self.runtime_handle.spawn(async move {
            _ = plugin.log(log_level.into(), target, message).await;
        });
    }

    fn flush(&self) {}
}
