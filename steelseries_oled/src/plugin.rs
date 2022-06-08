use std::collections::HashMap;
use std::ffi::c_void;
use std::ptr::null_mut;

use libloading::Library;
use serde_json::Value;

use common_rs::interface::{
    Context, DisplayNameFn, FinalizeFn, InitializeFn, ManagedString, TypesFn, UpdateFn, StatusCode
};

pub struct Plugin {
    _lib: Library,
    ctx: *mut c_void,
    display_name_fn: DisplayNameFn,
    types_fn: TypesFn,
    update_fn: UpdateFn,
    finalize_fn: FinalizeFn,
}

// #[derive(Serialize, Deserialize, Debug)]
// #[serde(transparent)]
// pub struct Types {
//     pub types: HashMap<String, String>
// }

impl Plugin {
    pub fn new(path: &String) -> Result<Plugin, Box<dyn std::error::Error>> {
        unsafe {
            let library = Library::new(path)?;
            let initialize_fn: InitializeFn = *library.get(b"initialize")?;
            let display_name_fn: DisplayNameFn = *library.get(b"display_name")?;
            let types_fn: TypesFn = *library.get(b"types")?;
            let update_fn: UpdateFn = *library.get(b"update")?;
            let finalize_fn: FinalizeFn = *library.get(b"finalize")?;

            let mut ctx: Context = null_mut();
            let ctx_handle: *mut Context = &mut ctx;

            match initialize_fn(ctx_handle) {
                StatusCode::Ok => Ok(Plugin {
                    _lib: library,
                    ctx,
                    display_name_fn,
                    types_fn,
                    update_fn,
                    finalize_fn,
                }),
                code => Err(format!("Init failed (code: {:?})", code))?
            }
        }
    }

    pub fn display_name(&self) -> String {
        let mut str = ManagedString::new();
        (self.display_name_fn)(self.ctx, &mut str);
        str.to_string()
    }

    pub fn types(&self) -> Option<HashMap<String, String>> {
        let mut str = ManagedString::new();
        (self.types_fn)(self.ctx, &mut str);
        return match str.len() {
            0 => None,
            _ => serde_json::from_str(str.to_str().unwrap()).unwrap()
        };
    }

    pub fn update(&self) -> Option<HashMap<String, Value>> {
        let mut str = ManagedString::new();
        (self.update_fn)(self.ctx, &mut str);
        return match str.len() {
            0 => None,
            _ => serde_json::from_str(str.to_str().unwrap()).unwrap()
        };
    }
}

impl Drop for Plugin {
    fn drop(&mut self) {
        (self.finalize_fn)(self.ctx);
    }
}