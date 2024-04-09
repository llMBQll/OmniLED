use mlua::{Lua, Value};

pub use crate::script_handler::script_data_types::Size;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    // DeviceNotSupported,
    // NameAlreadyRegistered(String),
    InitFailed(String),
    WrongParameter(String),
    // TemporarilyUnavailable,
    // Offline,
    // Custom(String),
}

pub trait Screen {
    fn init(lua: &Lua, settings: Value) -> Result<Self>
    where
        Self: Sized;

    fn size(&mut self, lua: &Lua) -> Result<Size>;

    fn update(&mut self, lua: &Lua, pixels: Vec<u8>) -> Result<()>;

    fn name(&mut self, lua: &Lua) -> Result<String>;

    // fn partial_update(&mut self, lua: &Lua, pixels: &Vec<u8>) -> Result<()>;
}

pub trait Settings {
    fn new(lua: &Lua, value: Value) -> Result<Self>
    where
        Self: Sized;
}
