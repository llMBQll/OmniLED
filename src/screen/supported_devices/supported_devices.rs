use log::{error, warn};
use mlua::{chunk, Lua, LuaSerdeExt, Table, Value};
use once_cell::sync::OnceCell;

use crate::screen::supported_devices::device_info::{OutputSettings, SteelseriesEngineDeviceSettings, USBDeviceSettings};

static OUTPUTS: OnceCell<Vec<OutputSettings>> = OnceCell::new();

pub fn get_supported_outputs() -> &'static Vec<OutputSettings> {
    OUTPUTS.get().expect("Call to load_supported_outputs has to be done before call to this function")
}

pub fn load_supported_outputs(lua: &Lua) {
    let supported_outputs = lua.create_table().unwrap();
    lua.globals().set("SUPPORTED_OUTPUTS", supported_outputs).unwrap();

    let load_usb_device = lua.create_function(load_usb_device).unwrap();
    let load_steelseries_engine_device = lua.create_function(load_steelseries_engine_device).unwrap();

    lua.load(chunk! {
        f, err = loadfile(SETTINGS.supported_outputs_file, "t", {
            usb_device = $load_usb_device,
            steelseries_engine_device = $load_steelseries_engine_device
        })
        if err then
            LOG.error("Failed to load the supported outputs file - " .. err)
            return
        end
        f()
    }).exec().unwrap();

    let supported_outputs: Table = lua.globals().get("SUPPORTED_OUTPUTS").unwrap();
    OUTPUTS.set(supported_outputs.pairs::<Value, Value>().map(|pair| -> OutputSettings {
        let (_, output) = pair.unwrap();
        lua.from_value(output).unwrap()
    }).collect()).unwrap();
}

// TODO make a generic function for all output types

fn load_usb_device(lua: &Lua, usb_device: Value) -> mlua::Result<()> {
    let output: USBDeviceSettings = match lua.from_value(usb_device.clone()) {
        Ok(usb_device) => usb_device,
        Err(err) => {
            error!("Failed to parse device data: {}", err);
            return Ok(());
        }
    };

    let supported_devices: Table = lua.globals().get("SUPPORTED_OUTPUTS").unwrap();
    match supported_devices.get::<_, Value>(output.name.clone()).unwrap() {
        Value::Nil => {
            let wrapper = lua.create_table().unwrap();
            wrapper.set("USBDevice", usb_device).unwrap();
            supported_devices.set(output.name.clone(), wrapper).unwrap()
        },
        _ => warn!("Device '{}' is already added", output.name)
    };

    Ok(())
}

fn load_steelseries_engine_device(lua: &Lua, steelseries_engine_device: Value) -> mlua::Result<()> {
    let output: SteelseriesEngineDeviceSettings = match lua.from_value(steelseries_engine_device.clone()) {
        Ok(steelseries_engine_device) => steelseries_engine_device,
        Err(err) => {
            error!("Failed to parse device data: {}", err);
            return Ok(());
        }
    };

    let supported_devices: Table = lua.globals().get("SUPPORTED_OUTPUTS").unwrap();
    match supported_devices.get::<_, Value>(output.name.clone()).unwrap() {
        Value::Nil => {
            let wrapper = lua.create_table().unwrap();
            wrapper.set("SteelseriesEngineDevice", steelseries_engine_device).unwrap();
            supported_devices.set(output.name.clone(), wrapper).unwrap()
        },
        _ => warn!("Device '{}' is already added", output.name)
    };

    Ok(())
}