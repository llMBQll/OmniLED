use mlua::{Lua, Value};

pub use crate::renderer::buffer::Buffer;
pub use crate::script_handler::script_data_types::MemoryRepresentation;
pub use crate::script_handler::script_data_types::Size;

pub trait Screen {
    fn init(lua: &Lua, settings: Value) -> mlua::Result<Self>
    where
        Self: Sized;

    fn size(&mut self, lua: &Lua) -> mlua::Result<Size>;

    fn update(&mut self, lua: &Lua, buffer: Buffer) -> mlua::Result<()>;

    fn name(&mut self, lua: &Lua) -> mlua::Result<String>;

    fn memory_representation(&mut self, lua: &Lua) -> mlua::Result<MemoryRepresentation>;

    // fn partial_update(&mut self, lua: &Lua, pixels: &Vec<u8>) -> Result<()>;
}

pub trait Settings {
    fn new(lua: &Lua, value: Value) -> mlua::Result<Self>
    where
        Self: Sized;
}
