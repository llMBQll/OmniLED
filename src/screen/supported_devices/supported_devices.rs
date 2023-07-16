use log::{error, warn};
use mlua::{chunk, Lua, LuaSerdeExt, Table, Value};
use once_cell::sync::OnceCell;

use crate::screen::supported_devices::device_info::DeviceInfo;

static DEVICES: OnceCell<Vec<DeviceInfo>> = OnceCell::new();

pub fn load_supported_devices(lua: &Lua) {
    let supported_devices = lua.create_table().unwrap();
    lua.globals().set("SUPPORTED_DEVICES", supported_devices).unwrap();

    let load_device = lua.create_function(|lua, device: Value| {
        let device_info: DeviceInfo = match lua.from_value(device.clone()) {
            Ok(device_info) => device_info,
            Err(err) => {
                error!("Failed to parse device data: {}", err);
                return Ok(());
            }
        };

        let supported_devices: Table = lua.globals().get("SUPPORTED_DEVICES").unwrap();
        match supported_devices.get::<_, Value>(device_info.name.clone()).unwrap() {
            Value::Nil => supported_devices.set(device_info.name, device).unwrap(),
            _ => warn!("Device '{}' is already added", device_info.name)
        };

        Ok(())
    }).unwrap();

    lua.load(chunk! {
        f, err = loadfile(SETTINGS.supported_devices_file, "t", { device = $load_device })
        if err then
            LOG.error("Failed to load the supported devices file - " .. err)
            return
        end
        f()
    }).exec().unwrap();

    let supported_devices: Table = lua.globals().get("SUPPORTED_DEVICES").unwrap();
    DEVICES.set(supported_devices.pairs::<Value, Value>().map(|pair| -> DeviceInfo {
        let (_, device_info) = pair.unwrap();
        lua.from_value(device_info).unwrap()
    }).collect()).unwrap();
}

pub fn get_supported_devices() -> &'static Vec<DeviceInfo> {
    DEVICES.get().unwrap()
}