use const_format::{map_ascii_case, Case};
use mlua::Lua;

use crate::model::operation::Bar;
use crate::model::operation::{Modifiers, Operation, Text};
use crate::model::rectangle::{Point, Rectangle, Size};

macro_rules! register_function {
    ($lua:ident, $table:ident, $func_name:ident) => {
        $table
            .set(
                map_ascii_case!(Case::Pascal, stringify!($func_name)),
                $lua.create_function($func_name).unwrap(),
            )
            .unwrap();
    };
}

pub fn load_operations(lua: &Lua) {
    let operations = lua.create_table().unwrap();

    register_function!(lua, operations, point);
    register_function!(lua, operations, size);
    register_function!(lua, operations, rectangle);
    register_function!(lua, operations, bar);
    register_function!(lua, operations, text);
    register_function!(lua, operations, modifiers);

    lua.globals().set("OPERATIONS", operations).unwrap();
}

fn point(_: &Lua, obj: Point) -> mlua::Result<Point> {
    Ok(obj)
}

fn size(_: &Lua, obj: Size) -> mlua::Result<Size> {
    Ok(obj)
}

fn rectangle(_: &Lua, obj: Rectangle) -> mlua::Result<Rectangle> {
    Ok(obj)
}

fn bar(_: &Lua, obj: Bar) -> mlua::Result<Operation> {
    Ok(Operation::Bar(obj))
}

fn text(_: &Lua, obj: Text) -> mlua::Result<Operation> {
    Ok(Operation::Text(obj))
}

fn modifiers(_: &Lua, obj: Modifiers) -> mlua::Result<Modifiers> {
    Ok(obj)
}
