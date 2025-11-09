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

#[cfg(target_os = "windows")]
pub mod semaphore {
    use std::time::Duration;
    use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT};
    use windows::Win32::System::Threading::{
        CreateSemaphoreW, ReleaseSemaphore, WaitForSingleObject,
    };
    use windows::core::PCWSTR;

    pub struct BinarySemaphore {
        handle: HANDLE,
    }

    impl BinarySemaphore {
        pub fn new(initial: bool) -> Self {
            let handle =
                unsafe { CreateSemaphoreW(None, initial as i32, 1, PCWSTR::null()).unwrap() };
            Self { handle }
        }

        pub fn try_acquire(&self) -> bool {
            self.try_acquire_for(Duration::ZERO)
        }

        pub fn try_acquire_for(&self, duration: Duration) -> bool {
            let duration_ms = duration.as_millis().clamp(0, u32::MAX as u128) as u32;
            unsafe {
                match WaitForSingleObject(self.handle, duration_ms) {
                    WAIT_OBJECT_0 => true,
                    WAIT_TIMEOUT => false,
                    error => panic!("{}", std::io::Error::from_raw_os_error(error.0 as i32)),
                }
            }
        }

        pub fn release(&self) {
            unsafe { ReleaseSemaphore(self.handle, 1, None).unwrap() };
        }
    }

    impl Drop for BinarySemaphore {
        fn drop(&mut self) {
            unsafe {
                _ = CloseHandle(self.handle);
            }
        }
    }

    unsafe impl Send for BinarySemaphore {}
    unsafe impl Sync for BinarySemaphore {}
}

#[cfg(target_os = "linux")]
pub mod semaphore {}
