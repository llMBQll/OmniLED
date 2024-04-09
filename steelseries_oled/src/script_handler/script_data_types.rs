use const_format::{map_ascii_case, Case};
use mlua::{ErrorContext, FromLua, Lua, UserData, UserDataFields};
use oled_derive::FromLuaTable;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize, FromLuaTable)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl UserData for Point {}

#[derive(Debug, Clone, Copy, Deserialize, FromLuaTable)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

impl UserData for Size {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("Width", |_, this| Ok(this.width));

        fields.add_field_method_get("Height", |_, this| Ok(this.height));
    }
}

#[derive(Debug, Clone, Copy, Deserialize, FromLuaTable)]
pub struct Rectangle {
    pub origin: Point,
    pub size: Size,
}

impl UserData for Rectangle {}

#[derive(Clone, Debug, FromLua)]
pub enum Operation {
    Bar(Bar),
    Text(Text),
}

impl UserData for Operation {}

#[derive(Clone, Debug, FromLuaTable)]
pub struct Bar {
    pub value: f32,
    pub position: Rectangle,

    #[mlua(default)]
    pub modifiers: Modifiers,
}

impl UserData for Bar {}

#[derive(Clone, Debug, FromLuaTable)]
pub struct Text {
    pub text: String,
    pub position: Rectangle,

    #[mlua(default)]
    pub modifiers: Modifiers,
}

impl UserData for Text {}

#[derive(Clone, Copy, Debug, Default, FromLuaTable)]
pub struct Modifiers {
    #[mlua(default(false))]
    pub flip_horizontal: bool,

    #[mlua(default(false))]
    pub flip_vertical: bool,

    #[mlua(default(false))]
    pub strict: bool,

    #[mlua(default(false))]
    pub vertical: bool,

    #[mlua(default(false))]
    pub scrolling: bool,

    pub font_size: Option<usize>,
}

impl UserData for Modifiers {}

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

pub fn load_script_data_types(lua: &Lua) {
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
