use log::error;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::{Config, Handle};
use std::path::Path;

pub fn init_with_level(file_path: impl AsRef<Path>, level_filter: log::LevelFilter) -> Handle {
    let config = create_config(file_path, level_filter);
    let handle = log4rs::init_config(config).unwrap();

    std::panic::set_hook(Box::new(|panic_info| {
        error!("{panic_info}");
        println!("{panic_info}");
    }));

    handle
}

pub fn init(file_path: impl AsRef<Path>) -> Handle {
    init_with_level(file_path, default_log_level())
}

pub fn change_log_level(
    handle: &Handle,
    file_path: impl AsRef<Path>,
    level_filter: log::LevelFilter,
) {
    let config = create_config(file_path, level_filter);
    handle.set_config(config);
}

#[cfg(debug_assertions)]
fn default_log_level() -> log::LevelFilter {
    log::LevelFilter::Debug
}

#[cfg(not(debug_assertions))]
fn default_log_level() -> log::LevelFilter {
    log::LevelFilter::Info
}

fn create_config(file_path: impl AsRef<Path>, level_filter: log::LevelFilter) -> Config {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[{d(%Y-%m-%d %H:%M:%S:%3f)}][{l}][{t}] {m}\n",
        )))
        .build(file_path)
        .unwrap();
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .logger(log4rs::config::Logger::builder().build("mio", log::LevelFilter::Error))
        .logger(log4rs::config::Logger::builder().build("hyper", log::LevelFilter::Error))
        .logger(log4rs::config::Logger::builder().build("rustls", log::LevelFilter::Error))
        .logger(log4rs::config::Logger::builder().build("tracing", log::LevelFilter::Error))
        .logger(log4rs::config::Logger::builder().build("ureq", log::LevelFilter::Error))
        .logger(log4rs::config::Logger::builder().build("warp", log::LevelFilter::Error))
        .build(Root::builder().appender("logfile").build(level_filter))
        .unwrap();

    config
}
