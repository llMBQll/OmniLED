use mlua::{LightUserData, UserData, UserDataMethods};

pub use crate::model::rectangle::Size;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    DeviceNotSupported,
    NameAlreadyRegistered(String),
    InitFailed(String),
    // TemporarilyUnavailable,
    // Offline,
    // Custom(String),
}

pub trait Screen {
    fn init(&mut self) -> Result<()>;

    fn size(&mut self) -> Result<Size>;

    fn update(&mut self, pixels: &Vec<u8>) -> Result<()>;

    fn name(&self) -> Result<String>;

    // fn partial_update(pixels: &Vec<u8>) -> Result<()>;
}