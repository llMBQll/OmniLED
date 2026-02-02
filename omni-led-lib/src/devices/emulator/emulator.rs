use mlua::{ErrorContext, FromLua, Lua, Value};
use omni_led_derive::FromLuaValue;

use crate::devices::device::{Buffer, Device, MemoryLayout, Settings as DeviceSettings, Size};
use crate::ui::window::Window;

pub struct Emulator {
    size: Size,
    name: String,
    window: Window,
}

impl Device for Emulator {
    fn init(lua: &Lua, settings: Value) -> mlua::Result<Self> {
        let settings = EmulatorSettings::from_lua(settings, lua)?;
        Ok(Self {
            size: settings.screen_size,
            name: settings.name.clone(),
            window: Window::open(settings.screen_size, settings.name),
        })
    }

    fn size(&mut self, _lua: &Lua) -> mlua::Result<Size> {
        Ok(self.size)
    }

    fn update(&mut self, _lua: &Lua, buffer: Buffer) -> mlua::Result<()> {
        self.window.update(buffer.bytes());
        Ok(())
    }

    fn name(&mut self, _lua: &Lua) -> mlua::Result<String> {
        Ok(self.name.clone())
    }

    fn memory_layout(&mut self, _lua: &Lua) -> mlua::Result<MemoryLayout> {
        Ok(MemoryLayout::BytePerPixel)
    }
}

#[derive(Clone, FromLuaValue)]
pub struct EmulatorSettings {
    screen_size: Size,
    name: String,
}

impl DeviceSettings for EmulatorSettings {
    type DeviceType = Emulator;

    fn name(&self) -> String {
        self.name.clone()
    }
}
