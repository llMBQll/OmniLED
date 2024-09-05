use convert_case::{Case, Casing};
use log::{debug, error};
use mlua::{chunk, Function, Lua, OwnedTable, Table, UserData, UserDataMethods, Value};
use oled_derive::UniqueUserData;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::common::common::exec_file;
use crate::common::scoped_value::ScopedValue;
use crate::common::user_data::{UniqueUserData, UserDataRef};
use crate::create_table_with_defaults;
use crate::screen::debug_output::debug_output::DebugOutput;
use crate::screen::screen::Screen;
use crate::screen::simulator::simulator::Simulator;
use crate::screen::steelseries_engine::steelseries_engine_device::SteelseriesEngineDevice;
use crate::screen::usb_device::usb_device::USBDevice;
use crate::settings::settings::{get_full_path, Settings};

type Constructor = fn(&Lua, Value) -> Box<dyn Screen>;

#[derive(UniqueUserData)]
pub struct Screens {
    screens: HashMap<String, ScreenEntry>,
    constructors: HashMap<String, Constructor>,
}

impl Screens {
    pub fn load(lua: &Lua) -> ScopedValue {
        let (constructors, env) = Self::create_loaders(lua);
        let screens = ScopedValue::new(lua, Self::identifier(), Self::new(constructors));
        Self::load_screens(lua, env);
        screens
    }

    pub fn screen_status(&self, name: &str) -> Option<ScreenStatus> {
        self.screens.get(name).map(|entry| match entry {
            ScreenEntry::Initializer(_) => ScreenStatus::Available,
            ScreenEntry::Loaded => ScreenStatus::Loaded,
        })
    }

    pub fn load_screen(&mut self, lua: &Lua, name: String) -> mlua::Result<Box<dyn Screen>> {
        let entry = self.screens.entry(name);
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
        self.screens.insert(name, ScreenEntry::Loaded);
        Ok(screen)
    }

    fn new(constructors: HashMap<String, Constructor>) -> Self {
        Self {
            screens: HashMap::new(),
            constructors,
        }
    }

    fn load_screens(lua: &Lua, env: Table) {
        let settings = UserDataRef::<Settings>::load(lua);
        exec_file(
            lua,
            &get_full_path(&settings.get().supported_screens_file),
            env,
        )
        .unwrap();
    }

    fn create_loaders(lua: &Lua) -> (HashMap<String, Constructor>, Table) {
        let mut constructors = HashMap::new();
        let env = create_table_with_defaults!(lua, {
            PLATFORM = PLATFORM,
        });

        let loaders = [
            Self::create_loader::<SteelseriesEngineDevice>(lua),
            Self::create_loader::<USBDevice>(lua),
            Self::create_loader::<DebugOutput>(lua),
            Self::create_loader::<Simulator>(lua),
        ];
        for (name, constructor, loader) in loaders {
            constructors.insert(name.clone(), constructor);
            env.set(name, loader).unwrap();
        }

        (constructors, env)
    }

    fn create_loader<T: Screen + 'static>(lua: &Lua) -> (String, Constructor, Function) {
        let type_name = std::any::type_name::<T>();
        let type_name = type_name.split("::").last().unwrap();
        let type_name = type_name.to_case(Case::Snake);

        let constructor: fn(&Lua, Value) -> Box<dyn Screen> = |lua, settings| {
            let mut screen = Box::new(T::init(lua, settings).unwrap());
            debug!("Initialized '{}'", screen.name(lua).unwrap());
            screen
        };

        let name = type_name.clone();
        let loader = lua
            .create_function(move |lua, settings: Table| {
                let screen_name: String = settings.get("name")?;
                let name = name.clone();
                lua.load(chunk! {
                    SCREENS:add_configuration($screen_name, $name, $settings)
                })
                .exec()
                .unwrap();

                Ok(())
            })
            .unwrap();

        (type_name, constructor, loader)
    }

    fn add_configuration(
        &mut self,
        name: String,
        kind: String,
        settings: Table,
    ) -> mlua::Result<()> {
        match self.screens.entry(name) {
            Entry::Occupied(entry) => {
                let message = format!(
                    "Screen configuration for '{}' is already registered",
                    entry.key()
                );
                error!("{}", message);
                return Err(mlua::Error::runtime(message));
            }
            Entry::Vacant(entry) => {
                let loader = self.constructors[&kind];
                entry.insert(ScreenEntry::Initializer(Initializer {
                    settings: settings.into_owned(),
                    constructor: loader,
                }));
            }
        }

        Ok(())
    }
}

impl UserData for Screens {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut(
            "add_configuration",
            |_lua, manager, (name, kind, settings): (String, String, Table)| {
                manager.add_configuration(name, kind, settings)
            },
        );
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

pub enum ScreenStatus {
    Available,
    Loaded,
}
