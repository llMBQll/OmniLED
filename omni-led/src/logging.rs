use log::{Level, LevelFilter, error};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::filter::{Filter, Response};
use log4rs::{Config, Handle};
use omni_led_lib::constants::constants::Constants;
use omni_led_lib::logging::logger::LogHandle;
use std::path::Path;
use std::sync::{Arc, RwLock};

pub struct OmniLedLogHandle {
    _handle: Handle,
    filter: DynamicFilter,
}

impl LogHandle for OmniLedLogHandle {
    fn set_level_filter(&self, level_filter: LevelFilter) {
        self.filter.set(level_filter);
    }
}

pub fn init() -> OmniLedLogHandle {
    let data_dir = Constants::data_dir();
    std::fs::create_dir_all(data_dir).unwrap();

    let path = Constants::data_dir().join("logging.log");
    let filter = DynamicFilter::new(default_log_level());
    let config = create_config(&path, filter.clone());
    let handle = log4rs::init_config(config).unwrap();

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        error!("{panic_info}");
        default_hook(panic_info);
    }));

    OmniLedLogHandle {
        _handle: handle,
        filter,
    }
}

fn create_config(file_path: impl AsRef<Path>, filter: DynamicFilter) -> Config {
    const FILE_APPENDER: &str = "file_appender";

    let file_appender = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[{d(%Y-%m-%d %H:%M:%S:%3f)}][{l}][{t}] {m}\n",
        )))
        .build(file_path)
        .unwrap();

    Config::builder()
        .appender(
            Appender::builder()
                .filter(Box::new(filter))
                .build(FILE_APPENDER, Box::new(file_appender)),
        )
        .build(
            Root::builder()
                .appender(FILE_APPENDER)
                .build(LevelFilter::Trace),
        )
        .unwrap()
}

#[cfg(debug_assertions)]
fn default_log_level() -> LevelFilter {
    LevelFilter::Debug
}

#[cfg(not(debug_assertions))]
fn default_log_level() -> LevelFilter {
    LevelFilter::Info
}

#[derive(Debug, Clone)]
struct DynamicFilter {
    level_filter: Arc<RwLock<LevelFilter>>,
}

impl DynamicFilter {
    pub fn new(level_filter: LevelFilter) -> Self {
        Self {
            level_filter: Arc::new(RwLock::new(level_filter)),
        }
    }

    pub fn set(&self, level_filter: LevelFilter) {
        *self.level_filter.write().unwrap() = level_filter;
    }

    #[inline]
    fn respond(target_level: Level, level_filter: LevelFilter) -> Response {
        if target_level > level_filter {
            return Response::Reject;
        } else {
            return Response::Accept;
        }
    }
}

impl Filter for DynamicFilter {
    fn filter(&self, record: &log::Record) -> Response {
        const TARGETS: &[&str] = &[
            // OmniLED implementation files
            "omni_led",
            "omni_led_api",
            "omni_led_lib",
            // Script files (+ 'script' as fallback if it failed to get script name)
            "devices.lua",
            "plugins.lua",
            "scripts.lua",
            "settings.lua",
            "script",
            // Plugin applications
            "plugin",
        ];

        for prefix in TARGETS {
            if record.target() == *prefix
                || record
                    .target()
                    .strip_prefix(*prefix)
                    .is_some_and(|prefix| prefix.starts_with("::"))
            {
                let level_filter = self.level_filter.read().unwrap();
                return Self::respond(record.level(), *level_filter);
            }
        }

        // Only allow error logging if target is not registered above
        return Self::respond(record.level(), LevelFilter::Error);
    }
}
