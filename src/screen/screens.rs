use std::collections::HashMap;
use std::ffi::c_void;
use mlua::{LightUserData, Lua, UserData, UserDataMethods};
use crate::screen::raw_usb::raw_usb::RawUSB;
use crate::screen::screen::Screen;
use crate::screen::supported_devices::supported_devices::load_supported_outputs;

pub struct Screens {
    screens: HashMap<String, Box<dyn Screen + Send>>,
}

impl Screens {
    pub fn load(lua: &Lua) {
        let this = Self {
            screens: HashMap::new(),
        };

        lua.globals().set("SCREENS", this).unwrap();
        load_supported_outputs(lua);
    }

    fn load_screen(name: String) -> Box<dyn Screen + Send> {
        // TODO add error handling and other creation options

        let screen = RawUSB::new(name).unwrap();
        Box::new(screen)
    }
}

impl UserData for Screens {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("load_screen", |_, this, key: String| {
            // TODO add error handling
            let screen = this.screens.entry(key.clone())
                .or_insert_with(|| Self::load_screen(key));

            Ok(LightUserData(screen as *mut _ as *mut c_void))
        });

        methods.add_method("update", |_, _, (screen, data): (LightUserData, Vec<u8>)| {
            // TODO Make movable without copying Rust -> Lua -> Rust: use LightUserData too?

            let screen = unsafe { &mut *(screen.0 as *mut Box<dyn Screen + Send>) };
            screen.update(&data).unwrap();
            Ok(())
        });

        methods.add_method("size", |_, _, screen: LightUserData| {
            let screen = unsafe { &mut *(screen.0 as *mut Box<dyn Screen + Send>) };
            match screen.size() {
                Ok(size) => Ok(Some(size)),
                Err(_) => Ok(None)
            }
        });
    }
}