use convert_case::{Case, Casing};
use log::{debug, error, log_enabled};
use mlua::{chunk, Function, Lua, Table, UserData, Value};
use omni_led_derive::UniqueUserData;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::common::common::exec_file;
use crate::common::user_data::{UniqueUserData, UserDataRef};
use crate::create_table_with_defaults;
use crate::devices::device::Device;
use crate::devices::emulator::emulator::Emulator;
use crate::devices::steelseries_engine::steelseries_engine_device::SteelseriesEngineDevice;
use crate::devices::usb_device::usb_device::USBDevice;
use crate::settings::settings::get_full_path;

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
        exec_file(lua, &get_full_path("devices.lua"), env).unwrap();
    }

    fn create_loaders(lua: &Lua) -> (HashMap<String, Constructor>, Table) {
        let mut constructors = HashMap::new();
        let env = create_table_with_defaults!(lua, {
            LOG = LOG,
            PLATFORM = PLATFORM,
        });

        let loaders = [
            Self::create_loader::<Emulator>(lua),
            Self::create_loader::<SteelseriesEngineDevice>(lua),
            Self::create_loader::<USBDevice>(lua),
        ];
        for (name, constructor, loader) in loaders {
            constructors.insert(name.clone(), constructor);
            env.set(name, loader).unwrap();
        }

        (constructors, env)
    }

    fn get_type_name<T: Device + 'static>() -> String {
        let type_name = std::any::type_name::<T>();
        type_name.split("::").last().unwrap().to_string()
    }

    fn create_loader<T: Device + 'static>(lua: &Lua) -> (String, Constructor, Function) {
        let constructor: fn(&Lua, Value) -> Box<dyn Device> = |lua, settings| {
            let mut device = Box::new(T::init(lua, settings).unwrap());

            if log_enabled!(log::Level::Debug) {
                let type_name = Self::get_type_name::<T>().to_case(Case::Snake);
                let device_name = device.name(lua).unwrap();
                debug!("Loaded {} '{}'", type_name, device_name);
            }

            device
        };

        let type_name = Self::get_type_name::<T>().to_case(Case::Snake);
        let function_name = type_name.clone();
        let loader = lua
            .create_function(move |lua, settings: Table| {
                let device_name: String = settings.get("name")?;
                let function_name = function_name.clone();

                let mut devices = UserDataRef::<Devices>::load(lua);
                devices
                    .get_mut()
                    .add_configuration(device_name, function_name, settings)
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
                let name = entry.key();
                debug!("Added config for {} '{}'", kind, name);

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

impl UserData for Devices {}

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
