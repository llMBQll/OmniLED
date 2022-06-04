use std::ffi::c_void;
use std::ptr::null_mut;

use libloading::Library;

use common_rs::managed_string::ManagedString;

type Context = *mut c_void;
type InitializeFn = fn(*mut Context) -> i32;
type UpdateFn = fn(Context, *mut ManagedString) -> i32;
type FinalizeFn = fn(Context) -> i32;

pub struct Plugin {
    _lib: Library,
    ctx: *mut c_void,
    update_fn: UpdateFn,
    finalize_fn: FinalizeFn,
}

impl Plugin {
    pub fn new(path: &String) -> Result<Plugin, Box<dyn std::error::Error>> {
        unsafe {
            let library = Library::new(path)?;
            let initialize_fn: InitializeFn = *library.get(b"initialize")?;
            let update_fn: UpdateFn = *library.get(b"update")?;
            let finalize_fn: FinalizeFn = *library.get(b"finalize")?;

            let mut ctx: Context = null_mut();
            let ctx_handle: *mut Context = &mut ctx;

            match initialize_fn(ctx_handle) {
                0 => Ok(Plugin {
                    _lib: library,
                    ctx,
                    update_fn,
                    finalize_fn,
                }),
                code => Err(format!("Init failed (code: {})", code))?
            }
        }
    }

    pub fn update(&self) {
        let mut str = ManagedString::new();
        (self.update_fn)(self.ctx, &mut str);
        println!("{}", &str);
    }
}

impl Drop for Plugin {
    fn drop(&mut self) {
        (self.finalize_fn)(self.ctx);
    }
}