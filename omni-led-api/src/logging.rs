/*
 * OmniLED is a software for displaying data on various OLED devices.
 * Copyright (C) 2024  Michał Bałabanow <m.balabanow@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use log::{LevelFilter, Log, Metadata, Record, error};
use tokio::runtime::Handle;
use tokio::sync::mpsc::Sender;

use crate::types::{LogData, LogLevel};

pub fn init(runtime_handle: Handle, log_sink: Sender<LogData>, log_level_filter: LevelFilter) {
    let logger = Logger::new(runtime_handle, log_sink, log_level_filter);
    log::set_boxed_logger(Box::new(logger))
        .map(|()| log::set_max_level(log_level_filter))
        .unwrap();

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        error!("{panic_info}");
        default_hook(panic_info);
    }));
}

struct Logger {
    runtime_handle: Handle,
    log_sink: Sender<LogData>,
    log_level_filter: LevelFilter,
}

impl Logger {
    pub fn new(
        runtime_handle: Handle,
        log_sink: Sender<LogData>,
        log_level_filter: LevelFilter,
    ) -> Self {
        Self {
            runtime_handle,
            log_sink,
            log_level_filter,
        }
    }
}

impl Log for Logger {
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
        self.runtime_handle
            .spawn(async move { log_sink.send(data).await.unwrap() });
    }

    fn flush(&self) {}
}
