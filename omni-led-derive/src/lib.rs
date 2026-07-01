pub use omni_led_derive_impl::*;

pub enum FromLuaError {
    Lua(mlua::Error),
    MissingFields,
}

impl From<mlua::Error> for FromLuaError {
    fn from(value: mlua::Error) -> Self {
        Self::Lua(value)
    }
}
