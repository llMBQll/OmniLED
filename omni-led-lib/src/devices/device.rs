use mlua::{FromLua, Lua, Value};

pub use crate::renderer::buffer::Buffer;
pub use crate::script_handler::script_data_types::{MemoryLayout, Size};

pub trait Device {
    fn init(lua: &Lua, settings: Value) -> mlua::Result<Self>
    where
        Self: Sized;

    fn size(&mut self, lua: &Lua) -> mlua::Result<Size>;

    fn update(&mut self, lua: &Lua, buffer: Buffer) -> mlua::Result<()>;

    fn name(&mut self, lua: &Lua) -> mlua::Result<String>;

    fn memory_layout(&mut self, lua: &Lua) -> mlua::Result<MemoryLayout>;
}

pub trait Settings: FromLua {
    type DeviceType: Device;

    fn name(&self) -> String;
}
