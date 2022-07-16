use std::{thread, thread::JoinHandle};
use std::sync::{Arc, Mutex};

use libloading::Library;

use common_rs::interface::{OnUpdateCallbackFn, RunFn, StatusCode};

pub struct Plugin {
    _lib: Library,
    run_fn: RunFn,
    running: i32,
    handle: Option<JoinHandle<StatusCode>>,
}

struct StatePtr {
    ptr: *const i32
}

impl StatePtr {
    pub fn new(ptr: *const i32) -> Self {
        Self {
            ptr
        }
    }
}

unsafe impl Send for StatePtr { }

impl Plugin {
    pub fn new(path: &String) -> Result<Plugin, Box<dyn std::error::Error>> {
        unsafe {
            let library = Library::new(path)?;
            let run_fn: RunFn = *library.get(b"run")?;

            Ok(Plugin {
                _lib: library,
                run_fn,
                running: 0,
                handle: None,
            })
        }
    }

    pub fn run(&mut self, on_update: OnUpdateCallbackFn) {
        self.running = 1;
        let ptr: Arc<Mutex<StatePtr>> = Arc::new(Mutex::new(StatePtr::new(&self.running)));
        let run_fn = self.run_fn;


        self.handle = Some(thread::spawn(move || {
            run_fn((*ptr.clone().lock().unwrap()).ptr, on_update)
        }));
    }

    pub fn stop(&mut self) -> Option<StatusCode> {
        self.running = 0;
        match self.handle.take() {
            Some(handle) => Some(handle.join().unwrap()),
            None => None
        }
    }
}
