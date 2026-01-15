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
pub mod semaphore {
    use std::{mem::MaybeUninit, time::Duration};

    use libc::{
        __errno_location, CLOCK_REALTIME, EAGAIN, EINTR, ETIMEDOUT, c_int, c_uint, clock_gettime,
        sem_destroy, sem_init, sem_post, sem_t, sem_timedwait, sem_trywait, time_t, timespec,
    };

    pub struct BinarySemaphore {
        sem: sem_t,
    }

    impl BinarySemaphore {
        pub fn new(initial: bool) -> Self {
            let mut sem = MaybeUninit::<sem_t>::zeroed();

            const PROCESS_LOCAL_SEM: c_int = 0;
            let result =
                unsafe { sem_init(sem.as_mut_ptr(), PROCESS_LOCAL_SEM, initial as c_uint) };
            if result != 0 {
                panic!("{}", std::io::Error::last_os_error());
            }

            Self {
                sem: unsafe { sem.assume_init() },
            }
        }

        pub fn try_acquire(&self) -> bool {
            let result = unsafe { sem_trywait(&self.sem as *const _ as *mut _) };
            if result != 0 {
                match unsafe { *__errno_location() } {
                    EAGAIN => false,
                    errno => {
                        panic!("{}", std::io::Error::from_raw_os_error(errno));
                    }
                }
            } else {
                true
            }
        }

        pub fn try_acquire_for(&self, duration: Duration) -> bool {
            let mut ts = timespec {
                tv_sec: 0,
                tv_nsec: 0,
            };

            let result = unsafe { clock_gettime(CLOCK_REALTIME, &mut ts as *mut _) };
            if result != 0 {
                panic!("{}", std::io::Error::last_os_error());
            }

            ts.tv_sec += duration.as_secs() as time_t;

            #[cfg(not(all(target_arch = "x86_64", target_pointer_width = "32")))]
            {
                use libc::c_long;
                ts.tv_nsec += duration.subsec_nanos() as c_long;
            }

            #[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
            {
                ts.tv_nsec += duration.subsec_nanos() as i64;
            }

            ts.tv_sec += ts.tv_nsec / 1_000_000_000;
            ts.tv_nsec = ts.tv_nsec % 1_000_000_000;

            unsafe {
                loop {
                    let wait_result =
                        sem_timedwait(&self.sem as *const _ as *mut _, &ts as *const _);
                    if wait_result != 0 {
                        match *__errno_location() {
                            EINTR => continue,
                            ETIMEDOUT => break false,
                            errno => {
                                panic!("{}", std::io::Error::from_raw_os_error(errno));
                            }
                        }
                    } else {
                        break true;
                    }
                }
            }
        }

        pub fn release(&self) {
            let result = unsafe { sem_post(&self.sem as *const _ as *mut _) };
            if result != 0 {
                panic!("{}", std::io::Error::last_os_error());
            }
        }
    }

    impl Drop for BinarySemaphore {
        fn drop(&mut self) {
            unsafe { sem_destroy(&self.sem as *const _ as *mut _) };
        }
    }

    unsafe impl Send for BinarySemaphore {}
    unsafe impl Sync for BinarySemaphore {}
}

#[cfg(target_os = "macos")]
pub mod semaphore {
    use std::sync::{Condvar, Mutex};
    use std::time::{Duration, Instant};

    pub struct BinarySemaphore {
        available: Mutex<bool>,
        cvar: Condvar,
    }

    impl BinarySemaphore {
        pub fn new(initial: bool) -> Self {
            Self {
                available: Mutex::new(initial),
                cvar: Condvar::new(),
            }
        }

        pub fn try_acquire(&self) -> bool {
            let mut available = self.available.lock().unwrap();
            if *available {
                *available = false;
                true
            } else {
                false
            }
        }

        pub fn try_acquire_for(&self, duration: Duration) -> bool {
            let deadline = Instant::now() + duration;
            let mut available = self.available.lock().unwrap();

            while !*available {
                let now = Instant::now();
                if now >= deadline {
                    return false;
                }

                let remaining = deadline - now;
                let (guard, timeout_result) = self.cvar.wait_timeout(available, remaining).unwrap();

                available = guard;

                if timeout_result.timed_out() && !*available {
                    return false;
                }
            }

            *available = false;
            true
        }

        pub fn release(&self) {
            let mut available = self.available.lock().unwrap();
            *available = true;
            self.cvar.notify_one();
        }
    }

    // impl Drop for BinarySemaphore {
    //     fn drop(&mut self) {
    //
    //     }
    // }

    // unsafe impl Send for BinarySemaphore {}
    // unsafe impl Sync for BinarySemaphore {}
}
