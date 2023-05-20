use std::marker::PhantomData;
use mlua::{Lua, Nil, Table, UserData, UserDataFields, UserDataMethods, Value};

pub use crate::model::position::Size;

pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    NameAlreadyRegistered(String),
    InitFailed(String),
    TemporarilyUnavailable,
    Offline,
    Custom(String),
}

pub trait Screen {
    fn init(&mut self) -> Result<()>;

    fn size(&mut self) -> Result<Size>;

    fn update(&mut self, pixels: &Vec<u8>) -> Result<()>;

    fn name(&self) -> Result<String>;

    // fn partial_update(pixels: &Vec<u8>) -> Result<()>;
}

fn register_screen<T: Send + Screen + 'static>(lua: &Lua, mut screen: T) -> Result<()> {
    screen.init()?;

    let name = screen.name()?;
    let screens: Table = lua.globals().get("SCREENS").unwrap();
    match screens.get::<_, Value>(name.clone()).unwrap() {
        Nil => {}
        _ => {
            return Err(Error::NameAlreadyRegistered(name));
        }
    }

    let wrapped = ScreenWrapper::new(screen);
    screens.set(name, wrapped).unwrap();

    Ok(())
}

struct ScreenWrapper<T: Send + Screen> {
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
        methods.add_method_mut("update", |_, this, data: i32| {
            todo!("Make data copiable Rust -> Lua -> Rust");
            Ok(())
        })
    }
}
