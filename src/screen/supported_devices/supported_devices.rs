use log::{error, warn};
use mlua::{chunk, Lua, LuaSerdeExt, Table, Value};
use once_cell::sync::OnceCell;

use crate::screen::supported_devices::device_info::{Output, USBDevice};

static OUTPUTS: OnceCell<Vec<Output>> = OnceCell::new();

pub fn get_supported_outputs() -> &'static Vec<Output> {
    OUTPUTS.get().expect("Call to load_supported_outputs has to be done before call to this function")
}

pub fn load_supported_outputs(lua: &Lua) {
    let supported_outputs = lua.create_table().unwrap();
    lua.globals().set("SUPPORTED_OUTPUTS", supported_outputs).unwrap();

    let load_usb_device = lua.create_function(load_usb_device).unwrap();

    lua.load(chunk! {
        f, err = loadfile(SETTINGS.supported_outputs_file, "t", { device = $load_usb_device })
        if err then
            LOG.error("Failed to load the supported outputs file - " .. err)
            return
        end
        f()
    }).exec().unwrap();

    let supported_outputs: Table = lua.globals().get("SUPPORTED_OUTPUTS").unwrap();
    OUTPUTS.set(supported_outputs.pairs::<Value, Value>().map(|pair| -> Output {
        let (_, output) = pair.unwrap();
        lua.from_value(output).unwrap()
    }).collect()).unwrap();
}

fn load_usb_device(lua: &Lua, device: Value) -> mlua::Result<()> {
    let output: USBDevice = match lua.from_value(device.clone()) {
        Ok(usb_device) => usb_device,
        Err(err) => {
            error!("Failed to parse device data: {}", err);
            return Ok(());
        }
    };

    let supported_devices: Table = lua.globals().get("SUPPORTED_OUTPUTS").unwrap();
    match supported_devices.get::<_, Value>(output.name.clone()).unwrap() {
        Value::Nil => {
            let output = lua.create_table().unwrap();
            output.set("USBDevice", device).unwrap();
            supported_devices.set(output.name, output).unwrap()
        },
        _ => warn!("Device '{}' is already added", output.name)
    };

    Ok(())
}

fn steelseries_engine_device(lua: &Lua, device: Value) -> mlua::Result<()> {

    Ok(())
}