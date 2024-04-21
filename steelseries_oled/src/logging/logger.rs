use log::{debug, error, info, trace, warn};
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config, Handle,
};
use mlua::{Lua, UserData, UserDataMethods};
use serde::de;
use serde::de::Error;

use crate::common::user_data::UserDataIdentifier;
use crate::constants::constants::Constants;

#[derive(Clone, Debug)]
pub struct Logger {
    handle: Handle,
}

impl Logger {
    pub fn load(lua: &Lua) {
        let config = Self::create_config(log::LevelFilter::Info);
        let handle = log4rs::init_config(config).unwrap();
        let logger = Logger { handle };

        lua.globals().set(Logger::identifier(), logger).unwrap();

        std::panic::set_hook(Box::new(|panic_info| {
            error!("{panic_info}");
            println!("{panic_info}");
        }));
    }

    pub fn set_level_filter(&self, level_filter: LevelFilter) {
        let config = Self::create_config(level_filter.0);
        self.handle.set_config(config);
    }

    fn create_config(level_filter: log::LevelFilter) -> Config {
        let logfile = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(
                "{t} [{d(%Y-%m-%d %H:%M:%S:%3f)}][{l}] {m}\n",
            )))
            .build(Constants::root_dir().join("logging.log"))
            .unwrap();

        let config = Config::builder()
            .appender(Appender::builder().build("logfile", Box::new(logfile)))
            .logger(log4rs::config::Logger::builder().build("mio", log::LevelFilter::Error))
            .logger(log4rs::config::Logger::builder().build("hyper", log::LevelFilter::Error))
            .logger(log4rs::config::Logger::builder().build("tracing", log::LevelFilter::Error))
            .logger(log4rs::config::Logger::builder().build("warp", log::LevelFilter::Error))
            .logger(log4rs::config::Logger::builder().build("ureq", log::LevelFilter::Error))
            .build(Root::builder().appender("logfile").build(level_filter))
            .unwrap();

        config
    }
}

impl UserData for Logger {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("debug", |_, _, message: String| {
            debug!("{}", message);
            Ok(())
        });

        methods.add_method("error", |_, _, message: String| {
            error!("{}", message);
            Ok(())
        });

        methods.add_method("info", |_, _, message: String| {
            info!("{}", message);
            Ok(())
        });
        methods.add_method("trace", |_, _, message: String| {
            trace!("{}", message);
            Ok(())
        });

        methods.add_method("warn", |_, _, message: String| {
            warn!("{}", message);
            Ok(())
        });
    }
}

#[derive(serde::Deserialize, Debug, Clone, Copy)]
pub struct LevelFilter(#[serde(deserialize_with = "deserialize_log_level")] pub log::LevelFilter);

fn deserialize_log_level<'de, D>(deserializer: D) -> Result<log::LevelFilter, D::Error>
where
    D: de::Deserializer<'de>,
{
    const NAMES: &[&str] = &["Off", "Error", "Warn", "Info", "Debug", "Trace"];

    let s: String = de::Deserialize::deserialize(deserializer)?;
    match s.as_str() {
        "Off" => Ok(log::LevelFilter::Off),
        "Error" => Ok(log::LevelFilter::Error),
        "Warn" => Ok(log::LevelFilter::Warn),
        "Info" => Ok(log::LevelFilter::Info),
        "Debug" => Ok(log::LevelFilter::Debug),
        "Trace" => Ok(log::LevelFilter::Trace),
        value => Err(Error::unknown_variant(value, NAMES)),
    }
}

impl UserDataIdentifier for Logger {
    fn identifier() -> &'static str {
        "LOG"
    }
}
