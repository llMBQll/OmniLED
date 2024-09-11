use convert_case::{Case, Casing};
use log::{debug, error};
use mlua::{chunk, Function, Lua, Table, UserData, UserDataMethods, Value};
use oled_derive::UniqueUserData;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::common::common::exec_file;
use crate::common::user_data::{UniqueUserData, UserDataRef};
use crate::create_table_with_defaults;
use crate::devices::device::Device;
use crate::devices::simulator::simulator::Simulator;
use crate::devices::steelseries_engine::steelseries_engine_device::SteelseriesEngineDevice;
use crate::devices::terminal::terminal::Terminal;
use crate::devices::usb_device::usb_device::USBDevice;
use crate::settings::settings::{get_full_path, Settings};

type Constructor = fn(&Lua, Value) -> Box<dyn Device>;

#[derive(UniqueUserData)]
pub struct Devices {
    devices: HashMap<String, DeviceEntry>,
    constructors: HashMap<String, Constructor>,
}

impl Devices {
    pub fn load(lua: &Lua) {
        let (constructors, env) = Self::create_loaders(lua);
        Self::set_unique(lua, Self::new(constructors));
        Self::load_devices(lua, env);
    }

    pub fn device_status(&self, name: &str) -> Option<DeviceStatus> {
        self.devices.get(name).map(|entry| match entry {
            DeviceEntry::Initializer(_) => DeviceStatus::Available,
            DeviceEntry::Loaded => DeviceStatus::Loaded,
        })
    }

    pub fn load_device(&mut self, lua: &Lua, name: String) -> mlua::Result<Box<dyn Device>> {
        let entry = self.devices.entry(name);
        let entry = match entry {
            Entry::Occupied(entry) => entry,
            Entry::Vacant(entry) => {
                return Err(mlua::Error::runtime(format!(
                    "Device {} not found",
                    entry.key()
                )));
            }
        };
        let name = entry.key().clone();
        let entry = entry.remove();
        let device = match entry {
            DeviceEntry::Initializer(initializer) => {
                let value = Value::Table(initializer.settings);
                let device = (initializer.constructor)(lua, value);
                device
            }
            DeviceEntry::Loaded => {
                return Err(mlua::Error::runtime(format!(
                    "Device {} was already loaded",
                    name
                )));
            }
        };
        self.devices.insert(name, DeviceEntry::Loaded);
        Ok(device)
    }

    fn new(constructors: HashMap<String, Constructor>) -> Self {
        Self {
            devices: HashMap::new(),
            constructors,
        }
    }

    fn load_devices(lua: &Lua, env: Table) {
        let settings = UserDataRef::<Settings>::load(lua);
        exec_file(lua, &get_full_path(&settings.get().devices_file), env).unwrap();
    }

    fn create_loaders(lua: &Lua) -> (HashMap<String, Constructor>, Table) {
        let mut constructors = HashMap::new();
        let env = create_table_with_defaults!(lua, {
            PLATFORM = PLATFORM,
        });

        let loaders = [
            Self::create_loader::<Simulator>(lua),
            Self::create_loader::<SteelseriesEngineDevice>(lua),
            Self::create_loader::<Terminal>(lua),
            Self::create_loader::<USBDevice>(lua),
        ];
        for (name, constructor, loader) in loaders {
            constructors.insert(name.clone(), constructor);
            env.set(name, loader).unwrap();
        }

        (constructors, env)
    }

    fn create_loader<T: Device + 'static>(lua: &Lua) -> (String, Constructor, Function) {
        let type_name = std::any::type_name::<T>();
        let type_name = type_name.split("::").last().unwrap();
        let type_name = type_name.to_case(Case::Snake);

        let constructor: fn(&Lua, Value) -> Box<dyn Device> = |lua, settings| {
            let mut device = Box::new(T::init(lua, settings).unwrap());
            debug!("Initialized '{}'", device.name(lua).unwrap());
            device
        };

        let function_name = type_name.clone();
        let loader = lua
            .create_function(move |lua, settings: Table| {
                let device_name: String = settings.get("name")?;
                let function_name = function_name.clone();
                lua.load(chunk! {
                    DEVICES:add_configuration($device_name, $function_name, $settings)
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
        match self.devices.entry(name) {
            Entry::Occupied(entry) => {
                let message = format!(
                    "Device configuration for '{}' is already registered",
                    entry.key()
                );
                error!("{}", message);
                return Err(mlua::Error::runtime(message));
            }
            Entry::Vacant(entry) => {
                let loader = self.constructors[&kind];
                entry.insert(DeviceEntry::Initializer(Initializer {
                    settings,
                    constructor: loader,
                }));
            }
        }

        Ok(())
    }
}

impl UserData for Devices {
    fn add_methods<'lua, M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut(
            "add_configuration",
            |_lua, manager, (name, kind, settings): (String, String, Table)| {
                manager.add_configuration(name, kind, settings)
            },
        );
    }
}

enum DeviceEntry {
    Initializer(Initializer),
    Loaded,
}

struct Initializer {
    settings: Table,
    constructor: fn(&Lua, Value) -> Box<dyn Device>,
}

pub enum DeviceStatus {
    Available,
    Loaded,
}
