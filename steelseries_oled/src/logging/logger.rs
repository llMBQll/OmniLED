use crate::constants::constants::Constants;
use log::{debug, error, info, trace, warn, LevelFilter};
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    init_config, Config, Handle,
};
use mlua::{Lua, UserData, UserDataMethods};

#[derive(Clone, Debug)]
pub struct Logger {
    _handle: Handle,
}

impl Logger {
    pub fn new(lua: &Lua) -> Logger {
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
                    .build(LevelFilter::Debug),
            )
            .unwrap();

        let handle = init_config(config).unwrap();
        let logger = Logger { _handle: handle };

        lua.globals().set("LOG", logger.clone()).unwrap();

        logger
    }
}

impl UserData for Logger {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("debug", |_, message: String| {
            debug!("{}", message);
            Ok(())
        });
        methods.add_function("error", |_, message: String| {
            error!("{}", message);
            Ok(())
        });
        methods.add_function("info", |_, message: String| {
            info!("{}", message);
            Ok(())
        });
        methods.add_function("trace", |_, message: String| {
            trace!("{}", message);
            Ok(())
        });
        methods.add_function("warn", |_, message: String| {
            warn!("{}", message);
            Ok(())
        });
    }
}
