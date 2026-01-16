use mlua::Lua;
use num_traits::Unsigned;

pub fn from_hex<T: Unsigned>(hex_value: String, _lua: &Lua) -> mlua::Result<T> {
    const HEX_PREFIX: &str = "0x";

    if !hex_value.starts_with(HEX_PREFIX) {
        return Err(mlua::Error::runtime(format!(
            "Hex number shall have a {HEX_PREFIX} prefix"
        )));
    }

    T::from_str_radix(&hex_value[2..], 16).map_err(move |_err| {
        mlua::Error::runtime(format!("Could not parse {} as hex value", hex_value))
    })
}
