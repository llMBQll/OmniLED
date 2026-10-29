use linkme::distributed_slice;
use mlua::{Lua, Table};

#[distributed_slice]
pub static LUA_ENUMS: [fn(&Lua, &Table) -> mlua::Result<()>];

#[distributed_slice]
pub static LUA_TYPES: [fn(&Lua, &Table) -> mlua::Result<()>];

pub fn set_lua_enums(lua: &Lua, env: &Table) {
    for lua_enum in LUA_ENUMS {
        lua_enum(lua, env).unwrap();
    }
}

pub fn set_lua_types(lua: &Lua, env: &Table) {
    for lua_type in LUA_TYPES {
        lua_type(lua, env).unwrap();
    }
}

#[macro_export]
macro_rules! register_lua_type {
    ($ty:ty) => {
        const _: () = {
            #[linkme::distributed_slice(crate::common::lua_register::LUA_TYPES)]
            static LUA_TYPE: fn(&mlua::Lua, &mlua::Table) -> mlua::Result<()> =
                <$ty as crate::common::lua_traits::LuaTypeStaticMembers>::register_members;
        };
    };
}
