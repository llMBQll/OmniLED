use minifb::{Window, WindowOptions};
use mlua::{ErrorContext, FromLua, Lua, Value};
use oled_derive::FromLuaValue;
use std::cmp::max;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::common::user_data::UserDataRef;
use crate::screen::screen::{Buffer, MemoryRepresentation, Screen, Size};
use crate::settings::settings::Settings;

pub struct Simulator {
    size: Size,
    name: String,
    buffer: Arc<Mutex<Vec<u32>>>,
    should_update: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    window_thread_handle: Option<JoinHandle<()>>,
}

#[derive(Clone, FromLuaValue)]
struct SimulatorSettings {
    screen_size: Size,
    name: String,
}

impl Screen for Simulator {
    fn init(lua: &Lua, settings: Value) -> mlua::Result<Self> {
        let settings = SimulatorSettings::from_lua(settings, lua)?;

        let size = settings.screen_size;
        let name = settings.name.clone();
        let buffer = vec![0; size.width * size.height];
        let buffer = Arc::new(Mutex::new(buffer));
        let running = Arc::new(AtomicBool::new(true));
        let should_update = Arc::new(AtomicBool::new(true));
        let update_interval = UserDataRef::<Settings>::load(lua).get().update_interval;

        let handle = thread::spawn({
            let buffer = Arc::clone(&buffer);
            let running = Arc::clone(&running);
            let should_update = Arc::clone(&should_update);
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

                let second = Duration::from_secs(1).as_millis() as usize;
                let update_interval = update_interval.as_millis() as usize;
                let target_fps = max(1, second / update_interval);
                window.set_target_fps(target_fps);

                while window.is_open() && running.load(Ordering::Relaxed) {
                    let update = should_update.swap(false, Ordering::Relaxed);
                    if !update {
                        continue;
                    }

                    window
                        .update_with_buffer(&buffer.lock().unwrap(), width, height)
                        .unwrap();
                }
            }
        });

        Ok(Self {
            size,
            name,
            buffer,
            should_update,
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
        self.should_update.store(true, Ordering::Relaxed);

        Ok(())
    }

    fn name(&mut self, _lua: &Lua) -> mlua::Result<String> {
        Ok(self.name.clone())
    }

    fn memory_representation(&mut self, _lua: &Lua) -> mlua::Result<MemoryRepresentation> {
        Ok(MemoryRepresentation::BytePerPixel)
    }
}

impl Drop for Simulator {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.window_thread_handle.take().map(JoinHandle::join);
    }
}
