use crate::OmniLedEvent;
use crate::common::common::get_proxy;
use crate::devices::device::{Buffer, Device, MemoryLayout, Settings as DeviceSettings, Size};
use crate::semaphore::semaphore::BinarySemaphore;
use mlua::{ErrorContext, FromLua, Lua, Value};
use omni_led_derive::FromLuaValue;
use std::cell::UnsafeCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use winit::event_loop::EventLoopProxy;

pub struct Emulator {
    size: Size,
    name: String,
    running: Arc<AtomicBool>,
    draw_buffer: Arc<DrawBuffer>,
    data_ready: Arc<BinarySemaphore>,
    reader_ready: Arc<BinarySemaphore>,
    proxy: EventLoopProxy<OmniLedEvent>,
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

        let proxy = get_proxy(&lua);
        proxy
            .send_event(OmniLedEvent::NewScreen(EmulatorHandle {
                height: size.height,
                width: size.width,
                name: name.clone(),
                draw_buffer: Arc::clone(&draw_buffer),
                data_ready: Arc::clone(&data_ready),
                reader_ready: Arc::clone(&reader_ready),
            }))
            .map_err(mlua::Error::external)?;

        Ok(Self {
            size,
            name,
            running,
            draw_buffer,
            data_ready,
            reader_ready,
            proxy,
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
            let draw_buffer: &mut Vec<u8> = unsafe {
                // SAFETY: `reader_ready` and `data_ready` semaphores make the threads run alternately,
                // so the buffer can be safely accessed.
                &mut *self.draw_buffer.data.get()
            };
            for (i, value) in buffer.bytes().iter().enumerate() {
                let index = i * 4;
                draw_buffer[index] = *value;
                draw_buffer[index + 1] = *value;
                draw_buffer[index + 2] = *value;
                draw_buffer[index + 3] = *value;
            }
            self.data_ready.release();

            _ = self.proxy.send_event(OmniLedEvent::Update);
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
    }
}

pub struct EmulatorHandle {
    pub height: usize,
    pub width: usize,
    pub name: String,
    draw_buffer: Arc<DrawBuffer>,
    data_ready: Arc<BinarySemaphore>,
    reader_ready: Arc<BinarySemaphore>,
}

impl EmulatorHandle {
    pub fn draw(&self, buffer: &mut [u8]) -> bool {
        let acquired = self.data_ready.try_acquire_for(Duration::from_millis(1));
        if acquired {
            let draw_buffer: &mut Vec<u8> = unsafe {
                // SAFETY: `reader_ready` and `data_ready` semaphores make the threads
                // run alternately, so the buffer can be safely accessed.
                &mut *self.draw_buffer.data.get()
            };
            buffer.copy_from_slice(draw_buffer);
            self.reader_ready.release();
        }
        acquired
    }
}

struct DrawBuffer {
    data: UnsafeCell<Vec<u8>>,
}

impl DrawBuffer {
    fn with_size(size: usize) -> Self {
        Self {
            data: UnsafeCell::new(vec![0; size * 4]),
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
