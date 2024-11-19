use crate::types::{LogData, LogLevel};
use log::{LevelFilter, Metadata, Record, Log};
use tokio::{sync::mpsc::Sender, task};

pub struct PluginLogger {
    log_sink: Sender<LogData>,
    log_level_filter: LevelFilter,
}

impl PluginLogger {
    pub fn new(log_sink: Sender<LogData>, log_level_filter: LevelFilter) -> Self {
        Self {
            log_sink,
            log_level_filter,
        }
    }
}

impl Log for PluginLogger {
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
