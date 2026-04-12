use mlua::{FromLuaMulti, Lua, Table, UserData, Value};

pub trait LuaName {
    const NAME: &str;
}

pub trait LuaConstructor: UserData + LuaName + 'static {
    type Args: FromLuaMulti;

    fn constructor(lua: &Lua, args: Self::Args) -> mlua::Result<Self>;

    fn register_constructor(lua: &Lua, env: &Table) -> mlua::Result<()> {
        let table = lua.create_table()?;
        table.set("new", lua.create_function(Self::constructor)?)?;
        env.set(Self::NAME, table)
    }
}

pub trait FromUserdata: UserData + LuaName + Clone + 'static {
    fn from_userdata(_lua: &Lua, value: Value) -> mlua::Result<Self> {
        match value {
            Value::UserData(userdata) => userdata.borrow::<Self>().map(|s| s.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: Self::NAME.to_string(),
                message: None,
            }),
        }
    }
}
