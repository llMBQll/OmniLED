use minifb::{Window, WindowOptions};
use mlua::{ErrorContext, FromLua, Lua, Value};
use oled_derive::FromLuaTable;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::screen::screen::{Screen, Size};

pub struct Simulator {
    size: Size,
    name: String,
    buffer: Arc<Mutex<Vec<u32>>>,
    should_update: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    window_thread_handle: Option<JoinHandle<()>>,
}

#[derive(Clone, FromLuaTable)]
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

                // update as often as incoming buffers
                // TODO load this value from lua environment, or from simulator specific settings
                window.limit_update_rate(Some(Duration::from_millis(50)));

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

    fn update(&mut self, _lua: &Lua, pixels: Vec<u8>) -> mlua::Result<()> {
        let width = self.size.width;
        let height = self.size.height;
        let len = width * height;
        let mut expanded = vec![0; len];
        for index in 0..len {
            // FIXME this calculation assumes no padding bits!
            let byte = index / 8;
            let bit = index % 8;
            let value = (pixels[byte] >> (7 - bit)) & 0b00000001;

            expanded[index] = value as u32 * 0xFFFFFF;
        }

        *self.buffer.lock().unwrap() = expanded;
        self.should_update.store(true, Ordering::Relaxed);

        Ok(())
    }

    fn name(&mut self, _lua: &Lua) -> mlua::Result<String> {
        Ok(self.name.clone())
    }
}

impl Drop for Simulator {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.window_thread_handle.take().map(JoinHandle::join);
    }
}
