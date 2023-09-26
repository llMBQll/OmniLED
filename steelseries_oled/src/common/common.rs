use log::error;
use mlua::{Lua, Table, TableExt, Value};

#[macro_export]
macro_rules! create_table {
    ($lua:ident, $values:tt) => {
        $lua.load(chunk! { $values }).eval::<mlua::Table>().unwrap()
    };
}

pub fn exec_file(lua: &Lua, name: &str, env: Table) {
    let (func, err): (Value, Value) = lua
        .globals()
        .call_function("loadfile", (name, "t", env))
        .unwrap();

    match (func, err) {
        (Value::Function(func), Value::Nil) => func.call::<_, ()>(()).unwrap(),
        (_, Value::String(err)) => error!("Error when running file: {}", err.to_str().unwrap()),
        _ => error!("Error when running file"),
    }
}
