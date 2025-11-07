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

use minifb::{Window, WindowOptions};
use mlua::{ErrorContext, FromLua, Lua, Value};
use omni_led_derive::FromLuaValue;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

use crate::common::user_data::UserDataRef;
use crate::devices::device::{Buffer, Device, MemoryLayout, Settings as DeviceSettings, Size};
use crate::settings::settings::Settings;

pub struct Emulator {
    size: Size,
    name: String,
    buffer: Arc<Mutex<Vec<u32>>>,
    running: Arc<AtomicBool>,
    window_thread_handle: Option<JoinHandle<()>>,
}

impl Device for Emulator {
    fn init(lua: &Lua, settings: Value) -> mlua::Result<Self> {
        let settings = EmulatorSettings::from_lua(settings, lua)?;

        let size = settings.screen_size;
        let name = settings.name.clone();
        let buffer = vec![0; size.width * size.height];
        let buffer = Arc::new(Mutex::new(buffer));
        let running = Arc::new(AtomicBool::new(true));
        let update_interval = UserDataRef::<Settings>::load(lua).get().update_interval;

        let handle = thread::spawn({
            let buffer = Arc::clone(&buffer);
            let running = Arc::clone(&running);
            move || {
                let width = size.width;
                let height = size.height;
                let name = settings.name;

                let mut window = Window::new(
                    &name,
                    width,
                    height,
                    WindowOptions {
                        resize: true,
                        ..Default::default()
                    },
                )
                .unwrap();

                while window.is_open() && running.load(Ordering::Relaxed) {
                    let begin = std::time::Instant::now();

                    window
                        .update_with_buffer(&buffer.lock().unwrap(), width, height)
                        .unwrap();

                    thread::sleep(update_interval.saturating_sub(begin.elapsed()));
                }
            }
        });

        Ok(Self {
            size,
            name,
            buffer,
            running,
            window_thread_handle: Some(handle),
        })
    }

    fn size(&mut self, _lua: &Lua) -> mlua::Result<Size> {
        Ok(self.size)
    }

    fn update(&mut self, _lua: &Lua, buffer: Buffer) -> mlua::Result<()> {
        let expanded = buffer
            .bytes()
            .iter()
            .map(|value| match value {
                0 => 0x000000,
                _ => 0xFFFFFF,
            })
            .collect();

        *self.buffer.lock().unwrap() = expanded;

        Ok(())
    }

    fn name(&mut self, _lua: &Lua) -> mlua::Result<String> {
        Ok(self.name.clone())
    }

    fn memory_layout(&mut self, _lua: &Lua) -> mlua::Result<MemoryLayout> {
        Ok(MemoryLayout::BytePerPixel)
    }
}

impl Drop for Emulator {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.window_thread_handle.take().map(JoinHandle::join);
    }
}

#[derive(Clone, FromLuaValue)]
pub struct EmulatorSettings {
    screen_size: Size,
    name: String,
}

impl DeviceSettings for EmulatorSettings {
    type DeviceType = Emulator;

    fn name(&self) -> String {
        self.name.clone()
    }
}
