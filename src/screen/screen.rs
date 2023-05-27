use mlua::{UserData,UserDataMethods};

pub use crate::model::rectangle::Size;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NameAlreadyRegistered(String),
    // InitFailed(String),
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

pub struct ScreenWrapper<T: Send + Screen> {
    screen: T,
}

impl<T: Send + Screen> ScreenWrapper<T> {
    pub fn new(screen: T) -> Self {
        Self {
            screen,
        }
    }
}

impl<T: Send + Screen> UserData for ScreenWrapper<T> {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("update", |_, this, data: Vec<u8>| {
            // TODO Make movable without copying Rust -> Lua -> Rust
            this.screen.update(&data).unwrap();
            Ok(())
        });

        methods.add_method_mut("size", |_, this, _: ()| {
            let size = match this.screen.size() {
                Ok(size) => size,
                Err(_) => Size { width: 0, height: 0 }
            };
            Ok(size)
        });
    }
}
