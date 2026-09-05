use log::error;
use log4rs::Config;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

use crate::constants::constants::Constants;
use crate::logging::dynamic_filter::DynamicFilter;
use crate::logging::level_filter_map::LogFilterMap;
use crate::logging::logger::{LevelFilter, LogImpl};

#[derive(Clone)]
pub struct FileLogger {
    filter: DynamicFilter,
}

static INSTANCE: OnceLock<FileLogger> = OnceLock::new();

impl FileLogger {
    pub fn instance() -> Self {
        INSTANCE.get_or_init(|| Self::init()).clone()
    }

    fn init() -> Self {
        let data_dir = Constants::data_dir();
        std::fs::create_dir_all(data_dir).unwrap();

        let path = Constants::data_dir().join("logging.log");
        let filter = DynamicFilter::new(LogFilterMap::default());
        let config = Self::create_config(&path, filter.clone());
        let _handle = log4rs::init_config(config).unwrap();

        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            error!("{panic_info}");
            default_hook(panic_info);
        }));

        Self { filter }
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
                    .build(log::LevelFilter::Trace),
            )
            .unwrap()
    }
}

impl LogImpl for FileLogger {
    fn set_filter_map(&self, filter_map: HashMap<String, LevelFilter>) {
        self.filter.set(filter_map);
    }
}
