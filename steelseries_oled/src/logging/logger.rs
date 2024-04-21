use log::{debug, error, info, trace, warn, LevelFilter};
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    init_config, Config, Handle,
};
use mlua::{Lua, UserData, UserDataMethods};

use crate::common::user_data::UserDataIdentifier;
use crate::constants::constants::Constants;

pub struct Logger {
    _handle: Handle,
}

impl Logger {
    pub fn load(lua: &Lua) {
        let logfile = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(
                "{t} [{d(%Y-%m-%d %H:%M:%S:%3f)}][{l}] {m}\n",
            )))
            .build(Constants::root_dir().join("logging.log"))
            .unwrap();

        let config = Config::builder()
            .appender(Appender::builder().build("logfile", Box::new(logfile)))
            .logger(log4rs::config::Logger::builder().build("mio", LevelFilter::Error))
            .logger(log4rs::config::Logger::builder().build("hyper", LevelFilter::Error))
            .logger(log4rs::config::Logger::builder().build("tracing", LevelFilter::Error))
            .logger(log4rs::config::Logger::builder().build("warp", LevelFilter::Error))
            .logger(log4rs::config::Logger::builder().build("ureq", LevelFilter::Error))
            .build(
                Root::builder()
                    .appender("logfile")
                    .build(Self::level_filter()),
            )
            .unwrap();

        let handle = init_config(config).unwrap();
        let logger = Logger { _handle: handle };

        lua.globals().set(Self::identifier(), logger).unwrap();

        std::panic::set_hook(Box::new(|panic_info| {
            error!("{panic_info}");
            println!("{panic_info}");
        }));
    }

    #[cfg(not(debug_assertions))]
    fn level_filter() -> LevelFilter {
        LevelFilter::Info
    }

    #[cfg(debug_assertions)]
    fn level_filter() -> LevelFilter {
        LevelFilter::Debug
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

impl UserDataIdentifier for Logger {
    fn identifier() -> &'static str {
        "LOG"
    }
}
