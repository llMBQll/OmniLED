use std::collections::HashMap;
use std::ffi::c_void;
use mlua::{LightUserData, Lua, LuaSerdeExt, Table, UserData, UserDataMethods, Value};

use crate::screen::screen::Screen;
use crate::screen::steelseries_engine::steelseries_engine_device::SteelseriesEngineDevice;
use crate::screen::supported_devices::device_info::OutputSettings;
use crate::screen::supported_devices::supported_devices::load_supported_outputs;
use crate::screen::usb_device::usb_device::USBDevice;

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

    fn load_screen(lua: &Lua, name: String) -> Box<dyn Screen + Send> {
        let supported_outputs: Table = lua.globals().get("SUPPORTED_OUTPUTS").unwrap();
        let output: Value = supported_outputs.get(name.clone()).unwrap();
        let output = lua.from_value::<OutputSettings>(output).expect("By this point all devices should be properly loaded");

        match output {
            OutputSettings::SteelseriesEngineDevice(x) => Box::new(SteelseriesEngineDevice::new(x)),
            OutputSettings::USBDevice(x) => Box::new(USBDevice::new(x).unwrap()), // TODO handle and log errors
        }
    }
}

impl UserData for Screens {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("load_screen", |lua, this, key: String| {
            // TODO add error handling
            let screen = this.screens.entry(key.clone())
                .or_insert_with(|| Self::load_screen(lua, key));

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