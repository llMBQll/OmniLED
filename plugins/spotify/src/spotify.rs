use std::time::{Duration, Instant};
use windows::{
    Win32::Foundation::{BOOL, HWND, LPARAM, HINSTANCE},
    Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextW, GetWindowThreadProcessId},
    Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
    Win32::System::ProcessStatus::K32GetModuleFileNameExW,
};

pub struct Spotify {
    hwnd: Option<HWND>,
    last_title: String,
    last_update: Instant,
}

impl Spotify {
    const INTERVAL: Duration = Duration::from_secs(1);

    pub fn new() -> Self {
        Self {
            hwnd: get_hwnd(),
            last_title: String::new(),
            last_update: Instant::now()
        }
    }

    pub fn update(&mut self) -> Option<(String, String)> {
        match self.hwnd {
            Some(hwnd) => {
                match get_title(hwnd) {
                    Some(title) => {
                        match title == self.last_title {
                            true => None,
                            false => {
                                self.last_title = title.to_string();
                                match title == "Spotify Premium" || title == "Spotify" {
                                    true => None,
                                    false => Some(split_title(title))
                                }
                            }
                        }
                    },
                    None => {
                        self.hwnd = None;
                        None
                    }
                }
            },
            None => {
                let now = Instant::now();
                if now.duration_since(self.last_update) > Self::INTERVAL {
                    self.hwnd = get_hwnd();
                    self.last_update = now;
                }
                None
            }
        }
    }
}

fn split_title(title: String) -> (String, String) {
    let parts = title.split(" - ");
    let mut iter = parts.into_iter();
    let artist = iter.next().unwrap();
    let track = iter.next().unwrap();
    (artist.to_string(), track.to_string())
}

fn get_title(hwnd: HWND) -> Option<String> {
    let mut buf: [u16; 256] = [0; 256];
    unsafe {
        let len = GetWindowTextW(hwnd, &mut buf);
        if len <= 0 {
            return None;
        }
        let len = len as usize;
        Some(String::from_utf16_lossy(&buf[..len as usize]))
    }
}

fn get_hwnd() -> Option<HWND> {
    let mut hwnd = HWND(0);
    unsafe {
        let hwnd = &mut hwnd as *mut HWND;
        EnumWindows(Some(enum_callback), LPARAM(hwnd as isize));
    }
    match hwnd {
        HWND(0) => None,
        hwnd => Some(hwnd) 
    }
}

extern "system" fn enum_callback(window: HWND, param: LPARAM) -> BOOL {
    unsafe {
        let mut buf: [u16; 256] = [0; 256];

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(window, &mut pid);
        let handle = match OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, true, pid) {
            Ok(handle) => handle,
            Err(_) => {
                return true.into();
            }
        };
        let name = match K32GetModuleFileNameExW(handle, HINSTANCE(0), &mut buf) {
            0..=11 => { return true.into(); }
            len => String::from_utf16_lossy(&buf[..len as usize])
        };

        if !name.ends_with("Spotify.exe") {
            return true.into();
        }

        let len = GetWindowTextW(window, &mut buf);
        let name = String::from_utf16_lossy(&buf[..len as usize]);
        match name.contains(" - ") || name == "Spotify Premium" || name == "Spotify" {
            true => {
                *(param.0 as *mut HWND) = window;
                false.into()
            }
            false => true.into()
        }
    }
}