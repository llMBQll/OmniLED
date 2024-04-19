use log::error;
use mlua::{chunk, Function, Lua, OwnedTable, Table, UserData, UserDataMethods, Value};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::common::common::exec_file;
use crate::common::scoped_value::ScopedValue;
use crate::create_table_with_defaults;
use crate::screen::debug_output::debug_output::DebugOutput;
use crate::screen::screen::Screen;
use crate::screen::steelseries_engine::steelseries_engine_device::SteelseriesEngineDevice;
use crate::screen::usb_device::usb_device::USBDevice;
use crate::settings::settings::{get_full_path, Settings};

pub struct Screens {
    screens: HashMap<String, ScreenEntry>,
    loaders: HashMap<String, fn(&Lua, Value) -> Box<dyn Screen>>,
}

impl Screens {
    pub fn load(lua: &Lua) -> ScopedValue {
        let screens = ScopedValue::new(lua, "SCREENS", Self::new());
        Self::load_screens(lua);
        screens
    }

    fn new() -> Self {
        let mut loaders: HashMap<String, fn(&Lua, Value) -> Box<dyn Screen>> = HashMap::new();

        loaders.insert("steelseries_engine_device".to_string(), |lua, settings| {
            Box::new(SteelseriesEngineDevice::init(lua, settings).unwrap())
        });

        loaders.insert("usb_device".to_string(), |lua, settings| {
            Box::new(USBDevice::init(lua, settings).unwrap())
        });

        loaders.insert("debug_output".to_string(), |lua, settings| {
            Box::new(DebugOutput::init(lua, settings).unwrap())
        });

        Self {
            screens: HashMap::new(),
            loaders,
        }
    }

    fn load_screens(lua: &Lua) {
        let load_steelseries_engine_device = Self::make_loader(lua, "steelseries_engine_device");
        let load_usb_device = Self::make_loader(lua, "usb_device");
        let load_debug_output = Self::make_loader(lua, "debug_output");

        let env = create_table_with_defaults!(lua, {
            steelseries_engine_device = $load_steelseries_engine_device,
            usb_device = $load_usb_device,
            debug_output = $load_debug_output,
            PLATFORM = PLATFORM,
        });

        exec_file(
            lua,
            &get_full_path(&Settings::get().supported_screens_file),
            env,
        )
        .unwrap();
    }

    fn make_loader<'a>(lua: &'a Lua, kind: &'static str) -> Function<'a> {
        lua.create_function(move |lua, settings: Table| {
            let name: String = settings.get("name")?;

            lua.load(chunk! {
                SCREENS:add_configuration($name, $kind, $settings)
            })
            .exec()
            .unwrap();

            Ok(())
        })
        .unwrap()
    }
}

impl UserData for Screens {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut(
            "add_configuration",
            |_lua, manager, (name, kind, settings): (String, String, Table)| {
                match manager.screens.entry(name) {
                    Entry::Occupied(entry) => {
                        let message = format!(
                            "Screen configuration for '{}' is already registered",
                            entry.key()
                        );
                        error!("{}", message);
                        return Err(mlua::Error::runtime(message));
                    }
                    Entry::Vacant(entry) => {
                        let loader = manager.loaders[&kind];
                        entry.insert(ScreenEntry::Initializer(Initializer {
                            settings: settings.into_owned(),
                            constructor: loader,
                        }));
                    }
                }

                Ok(())
            },
        );

        methods.add_method_mut("load", |lua, manager, name: String| {
            let entry = manager.screens.entry(name);
            let entry = match entry {
                Entry::Occupied(entry) => entry,
                Entry::Vacant(entry) => {
                    return Err(mlua::Error::runtime(format!(
                        "Screen {} not found",
                        entry.key()
                    )));
                }
            };
            let name = entry.key().clone();
            let entry = entry.remove();
            let screen = match entry {
                ScreenEntry::Initializer(initializer) => {
                    let value = Value::Table(initializer.settings.to_ref());
                    let screen = (initializer.constructor)(lua, value);
                    screen
                }
                ScreenEntry::Loaded => {
                    return Err(mlua::Error::runtime(format!(
                        "Screen {} was already loaded",
                        name
                    )));
                }
            };
            manager.screens.insert(name, ScreenEntry::Loaded);
            Ok(screen)
        });
    }
}

enum ScreenEntry {
    Initializer(Initializer),
    Loaded,
}

struct Initializer {
    settings: OwnedTable,
    constructor: fn(&Lua, Value) -> Box<dyn Screen>,
}
