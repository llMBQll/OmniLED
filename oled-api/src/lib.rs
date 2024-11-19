mod plugin {
    tonic::include_proto!("plugin");
}

pub use plugin::*;

use log::{Metadata, Record};
use tokio::sync::mpsc::{channel, Sender};
use tokio::task;
use tokio_stream::wrappers::ReceiverStream;

#[derive(Debug)]
pub struct Plugin {
    name: String,
    client: plugin_client::PluginClient<tonic::transport::Channel>,
}

impl Plugin {
    pub async fn new(name: &str, url: &str) -> Result<Self, tonic::transport::Error> {
        let mut client = plugin_client::PluginClient::connect(format!("http://{url}")).await?;

        let (tx, rx) = channel(128);
        let stream = ReceiverStream::new(rx);

        let log_level: log::LevelFilter = match client.log(stream).await {
            Ok(response) => response.into_inner().log_level_filter().into(),
            Err(_) => todo!(),
        };

        let logger = PluginLogger::new(tx, log_level);
        log::set_boxed_logger(Box::new(logger))
            .map(|()| log::set_max_level(log_level))
            .unwrap();

        Ok(Self {
            name: name.to_string(),
            client,
        })
    }

    pub async fn update_with_name(
        &mut self,
        name: &str,
        fields: Table,
    ) -> Result<(), tonic::Status> {
        let data = EventData {
            name: name.to_string(),
            fields: Some(fields),
        };

        self.client.event(data).await?;
        Ok(())
    }

    pub async fn update(&mut self, fields: Table) -> Result<(), tonic::Status> {
        let name = self.name.clone();

        self.update_with_name(&name, fields).await?;
        Ok(())
    }

    pub fn is_valid_identifier(identifier: &str) -> bool {
        if identifier.len() == 0 {
            return false;
        }

        let mut chars = identifier.chars();

        let first = chars.next().unwrap();
        if first != '_' && (first < 'A' || first > 'Z') {
            return false;
        }

        for c in chars {
            if c != '_' && (c < 'A' || c > 'Z') && (c < '0' || c > '9') {
                return false;
            }
        }

        true
    }
}

struct PluginLogger {
    log_sink: Sender<LogData>,
    log_level_filter: log::LevelFilter,
}

impl PluginLogger {
    pub fn new(log_sink: Sender<LogData>, log_level_filter: log::LevelFilter) -> Self {
        Self {
            log_sink,
            log_level_filter,
        }
    }
}

impl log::Log for PluginLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.log_level_filter
    }

    fn log(&self, record: &Record) {
        if !self.enabled(&record.metadata()) {
            return;
        }

        let log_level: LogLevel = record.level().into();
        let data = LogData {
            log_level: log_level as i32,
            location: record.target().to_string(),
            message: format!("{}", record.args()),
        };

        let log_sink = self.log_sink.clone();
        task::spawn(async move { log_sink.send(data).await.unwrap() });
    }

    fn flush(&self) {}
}

impl From<log::Level> for LogLevel {
    fn from(value: log::Level) -> Self {
        match value {
            log::Level::Error => LogLevel::Error,
            log::Level::Warn => LogLevel::Warn,
            log::Level::Info => LogLevel::Info,
            log::Level::Debug => LogLevel::Debug,
            log::Level::Trace => LogLevel::Trace,
        }
    }
}

impl Into<log::Level> for LogLevel {
    fn into(self) -> log::Level {
        match self {
            LogLevel::Unknown => todo!(),
            LogLevel::Error => log::Level::Error,
            LogLevel::Warn => log::Level::Warn,
            LogLevel::Info => log::Level::Info,
            LogLevel::Debug => log::Level::Debug,
            LogLevel::Trace => log::Level::Trace,
        }
    }
}

impl From<log::LevelFilter> for LogLevelFilter {
    fn from(value: log::LevelFilter) -> Self {
        match value {
            log::LevelFilter::Off => LogLevelFilter::Off,
            log::LevelFilter::Error => LogLevelFilter::Error,
            log::LevelFilter::Warn => LogLevelFilter::Warn,
            log::LevelFilter::Info => LogLevelFilter::Info,
            log::LevelFilter::Debug => LogLevelFilter::Debug,
            log::LevelFilter::Trace => LogLevelFilter::Trace,
        }
    }
}

impl Into<log::LevelFilter> for LogLevelFilter {
    fn into(self) -> log::LevelFilter {
        match self {
            LogLevelFilter::Unknown => todo!(),
            LogLevelFilter::Off => log::LevelFilter::Off,
            LogLevelFilter::Error => log::LevelFilter::Error,
            LogLevelFilter::Warn => log::LevelFilter::Warn,
            LogLevelFilter::Info => log::LevelFilter::Info,
            LogLevelFilter::Debug => log::LevelFilter::Debug,
            LogLevelFilter::Trace => log::LevelFilter::Trace,
        }
    }
}

macro_rules! cast_and_into_field {
    ($from:ty, $to:ty, $variant:expr) => {
        impl Into<Field> for $from {
            fn into(self) -> Field {
                Field {
                    field: Some($variant(self as $to)),
                }
            }
        }
    };
}

macro_rules! into_field {
    ($from:ty, $variant:expr) => {
        impl Into<Field> for $from {
            fn into(self) -> Field {
                Field {
                    field: Some($variant(self)),
                }
            }
        }
    };
}

// Boolean values
into_field!(bool, field::Field::FBool);

// Integer values
cast_and_into_field!(i8, i64, field::Field::FInteger);
cast_and_into_field!(i16, i64, field::Field::FInteger);
cast_and_into_field!(i32, i64, field::Field::FInteger);
into_field!(i64, field::Field::FInteger);
cast_and_into_field!(i128, i64, field::Field::FInteger);
cast_and_into_field!(u8, i64, field::Field::FInteger);
cast_and_into_field!(u16, i64, field::Field::FInteger);
cast_and_into_field!(u32, i64, field::Field::FInteger);
cast_and_into_field!(u64, i64, field::Field::FInteger);
cast_and_into_field!(u128, i64, field::Field::FInteger);

// Floating point values
cast_and_into_field!(f32, f64, field::Field::FFloat);
into_field!(f64, field::Field::FFloat);

// String values
into_field!(String, field::Field::FString);

impl Into<Field> for &str {
    fn into(self) -> Field {
        Field {
            field: Some(field::Field::FString(self.to_owned())),
        }
    }
}

// Array values
into_field!(Array, field::Field::FArray);

impl<T: Into<Field>> Into<Field> for Vec<T> {
    fn into(self) -> Field {
        let array = Array {
            items: self.into_iter().map(|entry| entry.into()).collect(),
        };

        array.into()
    }
}

// Image values
into_field!(Image, field::Field::FImage);
