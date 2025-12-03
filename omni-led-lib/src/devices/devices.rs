use convert_case::{Case, Casing};
use log::{debug, error, log_enabled};
use mlua::{Function, Lua, Table, UserData, Value, chunk};
use omni_led_derive::UniqueUserData;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::common::user_data::{UniqueUserData, UserDataRef};
use crate::constants::config::{ConfigType, load_config};
use crate::create_table_with_defaults;
use crate::devices::device::{Device, Settings};
use crate::devices::emulator::emulator::EmulatorSettings;
use crate::devices::steelseries_engine::steelseries_engine_device::SteelseriesEngineDeviceSettings;
use crate::devices::usb_device;
use crate::devices::usb_device::usb_device::USBDeviceSettings;
use crate::script_handler::script_data_types;

type Constructor = fn(&Lua, Value) -> Box<dyn Device>;

#[derive(UniqueUserData)]
pub struct Devices {
    devices: HashMap<String, DeviceEntry>,
    constructors: HashMap<String, Constructor>,
}

impl Devices {
    pub fn load(lua: &Lua, config: String) {
        let (constructors, env) = Self::create_loaders(lua);
        usb_device::transform::load_common_functions(lua, &env);
        script_data_types::load_script_data_types(lua, &env);
        Self::set_unique(lua, Self::new(constructors));
        load_config(lua, ConfigType::Devices, &config, env).unwrap();
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
                let device = (initializer.constructor)(lua, initializer.settings);
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

    fn create_loaders(lua: &Lua) -> (HashMap<String, Constructor>, Table) {
        let mut constructors = HashMap::new();
        let env = create_table_with_defaults!(lua, {
            LOG = LOG,
            PLATFORM = PLATFORM,
        });

        let loaders = [
            Self::create_loader::<EmulatorSettings>(lua),
            Self::create_loader::<SteelseriesEngineDeviceSettings>(lua),
            Self::create_loader::<USBDeviceSettings>(lua),
        ];

        for (name, constructor, loader) in loaders {
            constructors.insert(name.clone(), constructor);
            env.set(name, loader).unwrap();
        }

        (constructors, env)
    }

    fn get_type_name<T: Device>() -> String {
        let type_name = std::any::type_name::<T>();
        type_name.split("::").last().unwrap().to_string()
    }

    fn create_loader<S: Settings + 'static>(lua: &Lua) -> (String, Constructor, Function) {
        type DeviceType<S> = <S as Settings>::DeviceType;

        let constructor: Constructor = |lua, settings| {
            let mut device = Box::new(<DeviceType<S>>::init(lua, settings).unwrap());

            if log_enabled!(log::Level::Debug) {
                let type_name = Self::get_type_name::<DeviceType<S>>().to_case(Case::Snake);
                let device_name = device.name(lua).unwrap();
                debug!("Loaded {} '{}'", type_name, device_name);
            }

            device
        };

        let type_name = Self::get_type_name::<DeviceType<S>>().to_case(Case::Snake);
        let function_name = type_name.clone();
        let loader = lua
            .create_function(move |lua, settings: Value| {
                let settings_obj = S::from_lua(settings.clone(), lua)?;
                let device_name = settings_obj.name();
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
        settings: Value,
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
                let constructor = self.constructors[&kind];

                debug!("Added config for {} '{}'", kind, name);

                entry.insert(DeviceEntry::Initializer(Initializer {
                    settings,
                    constructor,
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
    settings: Value,
    constructor: Constructor,
}

pub enum DeviceStatus {
    Available,
    Loaded,
}
