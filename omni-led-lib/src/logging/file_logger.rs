use log::{Level, error};
use log4rs::Config;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::filter::{Filter, Response};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

use crate::constants::constants::Constants;
use crate::logging::logger::{LevelFilter, LogFilterMap, LogImpl};

pub struct FileLogger {
    filter: DynamicFilter,
}

impl LogImpl for FileLogger {
    fn set_filter_map(&self, filter_map: HashMap<String, LevelFilter>) {
        self.filter.set(filter_map);
    }
}

pub fn init() -> FileLogger {
    let data_dir = Constants::data_dir();
    std::fs::create_dir_all(data_dir).unwrap();

    let path = Constants::data_dir().join("logging.log");
    let filter = DynamicFilter::new(LogFilterMap::default());
    let config = create_config(&path, filter.clone());
    let _handle = log4rs::init_config(config).unwrap();

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        error!("{panic_info}");
        default_hook(panic_info);
    }));

    FileLogger { filter }
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

#[derive(Debug, Clone)]
struct DynamicFilter {
    filter_map: Arc<RwLock<HashMap<String, LevelFilter>>>,
}

impl DynamicFilter {
    pub fn new(filter_map: HashMap<String, LevelFilter>) -> Self {
        Self {
            filter_map: Arc::new(RwLock::new(filter_map)),
        }
    }

    pub fn set(&self, filter_map: HashMap<String, LevelFilter>) {
        *self.filter_map.write().unwrap() = filter_map;
    }

    #[inline]
    fn respond(target_level: Level, level_filter: LevelFilter) -> Response {
        if level_filter < target_level {
            return Response::Reject;
        } else {
            return Response::Accept;
        }
    }
}

impl Filter for DynamicFilter {
    fn filter(&self, record: &log::Record) -> Response {
        let filter_map = self.filter_map.read().unwrap();

        let mut target = record.target();
        loop {
            match filter_map.get(target) {
                Some(level_filter) => {
                    return Self::respond(record.level(), *level_filter);
                }
                None => match target.rfind("::") {
                    Some(index) => {
                        target = &target[..index];
                    }
                    None => {
                        break;
                    }
                },
            }
        }

        // Only allow error logging if target is not registered above
        return Self::respond(record.level(), LevelFilter::Error);
    }
}
