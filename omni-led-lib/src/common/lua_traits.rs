use mlua::{FromLuaMulti, IntoLua, IntoLuaMulti, Lua, Table, UserData, Value};

pub trait LuaName {
    const NAME: &str;
}

pub trait LuaTypeStaticMembers: UserData + LuaName + 'static {
    fn add_members(functions: &mut StaticMembers<'_>);

    fn register_members(lua: &Lua, env: &Table) -> mlua::Result<()> {
        let mut members = StaticMembers {
            lua,
            members: Vec::new(),
        };
        Self::add_members(&mut members);

        let table = lua.create_table()?;
        for (name, member) in members.members {
            table.set(name, member)?;
        }
        env.set(Self::NAME, table)
    }
}

pub struct StaticMembers<'a> {
    lua: &'a Lua,
    members: Vec<(String, Value)>,
}

impl<'a> StaticMembers<'a> {
    pub fn add_function<F, A, R>(&mut self, name: impl Into<String>, function: F)
    where
        F: Fn(&Lua, A) -> mlua::Result<R> + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti,
    {
        let function = self.lua.create_function(function).unwrap();
        self.add_member(name, function);
    }

    pub fn add_member(&mut self, name: impl Into<String>, value: impl IntoLua) {
        self.members
            .push((name.into(), value.into_lua(self.lua).unwrap()));
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
