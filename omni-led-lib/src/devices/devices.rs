use convert_case::{Case, Casing};
use log::{debug, error, log_enabled};
use mlua::{Function, Lua, UserData, Value};
use omni_led_derive::LuaName;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::common::lua_traits::LuaName;
use crate::common::sandbox::Sandbox;
use crate::common::user_data::{UserDataRef, set_unique_user_data};
use crate::constants::config::{ConfigType, load_config};
use crate::constants::constants::Constants;
use crate::devices::device::{Device, Settings as DeviceSettings};
use crate::devices::emulator::emulator::EmulatorSettings;
use crate::devices::steelseries_engine::steelseries_engine_device::SteelSeriesEngineDeviceSettings;
use crate::devices::usb_device;
use crate::devices::usb_device::hid_device::HidDeviceSettings;
use crate::devices::usb_device::raw_usb_device::RawUsbDeviceSettings;
use crate::logging::logger::Log;
use crate::sandbox_value;
use crate::settings::settings::Settings;

type Constructor = fn(&Lua, Value) -> mlua::Result<Box<dyn Device>>;

#[derive(LuaName)]
pub struct Devices {
    devices: HashMap<String, DeviceEntry>,
    constructors: HashMap<String, Constructor>,
}

impl Devices {
    pub fn load(lua: &Lua, config: String) {
        let (constructors, env) = Self::create_loaders(lua);
        usb_device::transform::load_common_functions(lua, &env.real);
        set_unique_user_data(lua, Self::new(constructors));
        load_config(lua, ConfigType::Devices, &config, env.proxy).unwrap();
    }

    pub fn load_device(&mut self, lua: &Lua, name: String) -> mlua::Result<Box<dyn Device>> {
        let entry = self.devices.entry(name.clone());
        match entry {
            Entry::Occupied(mut entry) => {
                let device_entry = entry.get_mut();

                if device_entry.available {
                    let constructor = device_entry.initializer.constructor;
                    let settings = device_entry.initializer.settings.clone();

                    let device = (constructor)(lua, settings)?;
                    device_entry.available = false;
                    Ok(device)
                } else {
                    Err(mlua::Error::runtime(format!(
                        "Device '{name}' was already loaded",
                    )))
                }
            }
            Entry::Vacant(entry) => Err(mlua::Error::runtime(format!(
                "Device '{}' not found",
                entry.key()
            ))),
        }
    }

    pub fn unload_device(&mut self, lua: &Lua, mut device: Box<dyn Device>) -> mlua::Result<()> {
        let name = device.name(lua)?;

        // Explicitly drop the device so it's more clear this function will infact unload it
        std::mem::drop(device);

        self.devices.entry(name.clone()).and_modify(|entry| {
            debug!("Unloaded device '{name}'");
            entry.available = true;
        });

        Ok(())
    }

    fn new(constructors: HashMap<String, Constructor>) -> Self {
        Self {
            devices: HashMap::new(),
            constructors,
        }
    }

    fn create_loaders(lua: &Lua) -> (HashMap<String, Constructor>, Sandbox) {
        let mut constructors = HashMap::new();
        let sandbox = Sandbox::new(
            lua,
            vec![
                sandbox_value!(Constants::NAME),
                sandbox_value!(Log::NAME),
                sandbox_value!(Settings::NAME),
            ],
        );

        let loaders = [
            Self::create_loader::<EmulatorSettings>(lua),
            Self::create_loader::<HidDeviceSettings>(lua),
            Self::create_loader::<RawUsbDeviceSettings>(lua),
            Self::create_loader::<SteelSeriesEngineDeviceSettings>(lua),
        ];

        for (name, constructor, loader) in loaders {
            constructors.insert(name.clone(), constructor);
            sandbox.real.set(name, loader).unwrap();
        }

        (constructors, sandbox)
    }

    fn get_type_name<T: Device>() -> String {
        let type_name = std::any::type_name::<T>();
        type_name.split("::").last().unwrap().to_string()
    }

    fn create_loader<S: DeviceSettings + 'static>(lua: &Lua) -> (String, Constructor, Function) {
        type DeviceType<S> = <S as DeviceSettings>::DeviceType;

        let constructor: Constructor = |lua, settings| {
            let mut device = Box::new(<DeviceType<S>>::init(lua, settings)?);

            if log_enabled!(log::Level::Debug) {
                let type_name = Self::get_type_name::<DeviceType<S>>().to_case(Case::Snake);
                let device_name = device.name(lua).unwrap();
                debug!("Loaded {} '{}'", type_name, device_name);
            }

            Ok(device)
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

                entry.insert(DeviceEntry {
                    initializer: Initializer {
                        settings,
                        constructor,
                    },
                    available: true,
                });
            }
        }

        Ok(())
    }
}

impl UserData for Devices {}

struct Initializer {
    settings: Value,
    constructor: Constructor,
}

struct DeviceEntry {
    initializer: Initializer,
    available: bool,
}
