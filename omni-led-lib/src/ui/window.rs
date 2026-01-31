use std::cell::UnsafeCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crate::devices::device::Size;
use crate::semaphore::semaphore::BinarySemaphore;
use crate::ui::event::Event;
use crate::ui::handler::{HandlerProxy, PROXY};

pub struct Window {
    proxy: HandlerProxy,
    id: Arc<AtomicU64>,
    draw_buffer: Arc<DrawBuffer>,
    data_ready: Arc<BinarySemaphore>,
    reader_ready: Arc<BinarySemaphore>,
}

impl Window {
    pub fn open(size: Size, name: String) -> Self {
        let proxy = PROXY.get().unwrap().clone();
        let id = Arc::new(AtomicU64::new(0));
        let draw_buffer = Arc::new(DrawBuffer::with_size(size));
        let data_ready = Arc::new(BinarySemaphore::new(false));
        let reader_ready = Arc::new(BinarySemaphore::new(true));

        let handle = WindowHandle {
            size,
            name: name.clone(),
            id: Arc::clone(&id),
            draw_buffer: Arc::clone(&draw_buffer),
            data_ready: Arc::clone(&data_ready),
            reader_ready: Arc::clone(&reader_ready),
        };

        proxy.send(Event::OpenWindow(handle));

        Self {
            proxy,
            id,
            draw_buffer,
            data_ready,
            reader_ready,
        }
    }

    pub fn update(&self, bytes: &[u8]) {
        // Only render if we don't have to wait for the emulator to be ready.
        // Skip the update otherwise, to not block the script engine thread.
        let acquired = self.reader_ready.try_acquire();
        if acquired {
            let draw_buffer: &mut Vec<u8> = unsafe {
                // SAFETY: `reader_ready` and `data_ready` semaphores make the threads run
                // alternately, so the buffer can be safely accessed.
                &mut *self.draw_buffer.data.get()
            };
            for i in 0..bytes.len() {
                let index = 4 * i;
                draw_buffer[index] = bytes[i];
                draw_buffer[index + 1] = bytes[i];
                draw_buffer[index + 2] = bytes[i];
                draw_buffer[index + 3] = bytes[i];
            }
            self.data_ready.release();

            self.proxy
                .send(Event::UpdateWindow(self.id.load(Ordering::Acquire)));
        }
    }
}

pub struct WindowHandle {
    pub size: Size,
    pub name: String,
    pub id: Arc<AtomicU64>,
    draw_buffer: Arc<DrawBuffer>,
    data_ready: Arc<BinarySemaphore>,
    reader_ready: Arc<BinarySemaphore>,
}

impl WindowHandle {
    pub fn draw(&self, buffer: &mut [u8]) {
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
    }
}

struct DrawBuffer {
    data: UnsafeCell<Vec<u8>>,
}

impl DrawBuffer {
    fn with_size(size: Size) -> Self {
        Self {
            data: UnsafeCell::new(vec![0; 4 * size.width * size.height]),
        }
    }
}

// SAFETY: This struct is only used in a single scenario where it is guarded by semaphores
unsafe impl Send for DrawBuffer {}

// SAFETY: This struct is only used in a single scenario where it is guarded by semaphores
unsafe impl Sync for DrawBuffer {}
