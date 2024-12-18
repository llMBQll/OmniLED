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

use log::error;
use mlua::{ErrorContext, FromLua, UserData};
use omni_led_derive::FromLuaValue;
use std::process::{Child, Command, Stdio};

pub struct Process {
    process: Child,
}

impl Process {
    pub fn new(config: &Config) -> std::io::Result<Process> {
        let mut command = Command::new(&config.path);
        let mut command = command
            .args(&config.args)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        Self::extra_configuration(&mut command);

        Ok(Self {
            process: command.spawn()?,
        })
    }

    #[cfg(target_os = "windows")]
    fn extra_configuration(command: &mut Command) {
        use std::os::windows::process::CommandExt;

        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    #[cfg(target_os = "linux")]
    fn extra_configuration(_command: &mut Command) {
        // No extra configuration required
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        match self.process.kill() {
            Ok(_) => {}
            Err(err) => {
                error!("{}", err.to_string())
            }
        }
    }
}

#[derive(Debug, Clone, FromLuaValue)]
pub struct Config {
    path: String,
    #[mlua(default)]
    args: Vec<String>,
}

impl UserData for Config {}
