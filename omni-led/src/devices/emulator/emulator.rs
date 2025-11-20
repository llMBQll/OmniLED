use minifb::{Window, WindowOptions};
use mlua::{ErrorContext, FromLua, Lua, Value};
use omni_led_derive::FromLuaValue;
use std::cell::UnsafeCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::devices::device::{Buffer, Device, MemoryLayout, Settings as DeviceSettings, Size};
use crate::semaphore::semaphore::BinarySemaphore;

pub struct Emulator {
    size: Size,
    name: String,
    running: Arc<AtomicBool>,
    window_thread_handle: Option<JoinHandle<()>>,
    draw_buffer: Arc<DrawBuffer>,
    data_ready: Arc<BinarySemaphore>,
    reader_ready: Arc<BinarySemaphore>,
}

impl Device for Emulator {
    fn init(lua: &Lua, settings: Value) -> mlua::Result<Self> {
        let settings = EmulatorSettings::from_lua(settings, lua)?;

        let size = settings.screen_size;
        let name = settings.name.clone();
        let running = Arc::new(AtomicBool::new(true));
        let draw_buffer = Arc::new(DrawBuffer::with_size(size.width * size.height));
        let data_ready = Arc::new(BinarySemaphore::new(false));
        let reader_ready = Arc::new(BinarySemaphore::new(true));

        let handle = thread::spawn({
            let running = Arc::clone(&running);
            let draw_buffer = Arc::clone(&draw_buffer);
            let data_ready = Arc::clone(&data_ready);
            let reader_ready = Arc::clone(&reader_ready);
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
                    // Wait for data to be ready, but can't try to acquire the lock indefinitely,
                    // as the application could be stopped or just stop updating the emulator.
                    // Wait up to 100 ms, then recheck the application state.
                    let acquired = data_ready.try_acquire_for(Duration::from_millis(100));

                    if acquired {
                        let draw_buffer: &mut Vec<u32> = unsafe {
                            // SAFETY: `reader_ready` and `data_ready` semaphores make the threads
                            // run alternately, so the buffer can be safely accessed.
                            &mut *draw_buffer.data.get()
                        };
                        window
                            .update_with_buffer(draw_buffer, width, height)
                            .unwrap();
                        reader_ready.release();
                    }
                }
            }
        });

        Ok(Self {
            size,
            name,
            running,
            window_thread_handle: Some(handle),
            draw_buffer,
            data_ready,
            reader_ready,
        })
    }

    fn size(&mut self, _lua: &Lua) -> mlua::Result<Size> {
        Ok(self.size)
    }

    fn update(&mut self, _lua: &Lua, buffer: Buffer) -> mlua::Result<()> {
        // Only render if we don't have to wait for the emulator to be ready.
        // Skip the update otherwise, to not block the main thread.
        let acquired = self.reader_ready.try_acquire();
        if acquired {
            let draw_buffer: &mut Vec<u32> = unsafe {
                // SAFETY: `reader_ready` and `data_ready` semaphores make the threads run alternately,
                // so the buffer can be safely accessed.
                &mut *self.draw_buffer.data.get()
            };
            for (i, value) in buffer.bytes().iter().enumerate() {
                draw_buffer[i] = match value {
                    0 => 0x000000,
                    _ => 0xFFFFFF,
                };
            }
            self.data_ready.release();
        }

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

struct DrawBuffer {
    data: UnsafeCell<Vec<u32>>,
}

impl DrawBuffer {
    fn with_size(size: usize) -> Self {
        Self {
            data: UnsafeCell::new(vec![0; size]),
        }
    }
}

// SAFETY: This struct is only used in a single scenario where it is guarded by semaphores
unsafe impl Send for DrawBuffer {}

// SAFETY: This struct is only used in a single scenario where it is guarded by semaphores
unsafe impl Sync for DrawBuffer {}

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
