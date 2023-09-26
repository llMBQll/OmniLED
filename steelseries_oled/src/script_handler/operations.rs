use mlua::{Lua, Table};

use crate::model::operation::Bar;
use crate::model::operation::{Modifiers, Operation, Text};
use crate::model::rectangle::{Point, Rectangle, Size};

pub fn load_operations(lua: &Lua) {
    let operations = lua.create_table().unwrap();

    operations
        .set("Point", lua.create_function(point).unwrap())
        .unwrap();
    operations
        .set("Size", lua.create_function(size).unwrap())
        .unwrap();
    operations
        .set("Rectangle", lua.create_function(rectangle).unwrap())
        .unwrap();
    operations
        .set("Bar", lua.create_function(bar).unwrap())
        .unwrap();
    operations
        .set("Text", lua.create_function(text).unwrap())
        .unwrap();
    operations
        .set("Modifiers", lua.create_function(modifiers).unwrap())
        .unwrap();

    lua.globals().set("OPERATIONS", operations).unwrap();
}

fn point(_: &Lua, args: Table) -> mlua::Result<Point> {
    let x = args.get("x")?;
    let y = args.get("y")?;

    Ok(Point { x, y })
}

fn size(_: &Lua, args: Table) -> mlua::Result<Size> {
    let width = args.get("width")?;
    let height = args.get("height")?;

    Ok(Size { width, height })
}

fn rectangle(_: &Lua, args: Table) -> mlua::Result<Rectangle> {
    let origin = args.get("origin")?;
    let size = args.get("size")?;

    Ok(Rectangle { origin, size })
}

fn bar(_: &Lua, args: Table) -> mlua::Result<Operation> {
    let value = args.get("value")?;
    let position = args.get("position")?;
    let modifiers = args.get("modifiers").unwrap_or(Modifiers::default());

    Ok(Operation::Bar(Bar {
        value,
        position,
        modifiers,
    }))
}

fn text(_: &Lua, args: Table) -> mlua::Result<Operation> {
    let text = args.get("text")?;
    let position = args.get("position")?;
    let modifiers = args.get("modifiers").unwrap_or(Modifiers::default());

    Ok(Operation::Text(Text {
        text,
        position,
        modifiers,
    }))
}

fn modifiers(_: &Lua, args: Table) -> mlua::Result<Modifiers> {
    let flip_horizontal = args.get("flip_horizontal").unwrap_or(false);
    let flip_vertical = args.get("flip_vertical").unwrap_or(false);
    let strict = args.get("strict").unwrap_or(false);
    let upper = args.get("upper").unwrap_or(false);
    let vertical = args.get("vertical").unwrap_or(false);
    let scrolling = args.get("scrolling").unwrap_or(false);

    Ok(Modifiers {
        flip_horizontal,
        flip_vertical,
        strict,
        upper,
        vertical,
        scrolling,
    })
}
