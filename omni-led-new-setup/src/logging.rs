/*
 * OmniLED is a software for displaying data on various OLED devices.
 * Copyright (C) 2025  Michał Bałabanow <m.balabanow@gmail.com>
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

use log::LevelFilter;
use omni_led_lib::logging::logger::LogHandle;
use simple_logger::SimpleLogger;

pub struct LogHandleImpl;

impl LogHandle for LogHandleImpl {
    fn set_level_filter(&self, _level_filter: LevelFilter) {
        // Do nothing
    }
}

pub fn init() {
    SimpleLogger::new()
        .with_module_level("wgpu", LevelFilter::Error)
        .with_module_level("cosmic", LevelFilter::Error)
        .with_module_level("iced", LevelFilter::Error)
        .with_module_level("naga", LevelFilter::Error)
        .init()
        .unwrap();
}
