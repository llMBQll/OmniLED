use convert_case::{Case, Casing};
use mlua::{ErrorContext, FromLua, Lua, Table, UserData, UserDataFields, Value};
use oled_derive::FromLuaTable;

#[derive(Debug, Clone, Copy, FromLuaTable)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl UserData for Point {}

#[derive(Debug, Clone, Copy, FromLuaTable)]
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

#[derive(Debug, Clone, Copy, FromLuaTable)]
pub struct Rectangle {
    pub origin: Point,
    pub size: Size,
}

impl UserData for Rectangle {}

#[derive(Debug, Clone, FromLuaTable)]
pub struct OledImage {
    pub size: Size,
    pub bytes: Vec<u8>,
}

impl UserData for OledImage {}

#[derive(Clone, Debug, FromLua)]
pub enum Operation {
    Bar(Bar),
    Image(Image),
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
pub struct Image {
    pub image: OledImage,
    pub position: Rectangle,

    #[mlua(default)]
    pub modifiers: Modifiers,
}

impl UserData for Image {}

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

#[derive(Clone, Copy)]
pub enum MemoryRepresentation {
    BitPerPixel,
    BytePerPixel,
}

impl<'lua> FromLua<'lua> for MemoryRepresentation {
    fn from_lua(value: Value<'lua>, _lua: &'lua Lua) -> mlua::Result<Self> {
        match value {
            Value::String(string) => {
                let string = string.to_string_lossy();
                match string.to_string().as_str() {
                    "BitPerPixel" => Ok(MemoryRepresentation::BitPerPixel),
                    "BytePerPixel" => Ok(MemoryRepresentation::BytePerPixel),
                    value => Err(mlua::Error::runtime(format!(
                        "Valid memory representations are ['BitPerPixel', 'BytePerPixel'], got '{}'",
                        value
                    ))),
                }
            }
            other => Err(mlua::Error::runtime(format!(
                "Expected a string, got '{}'",
                other.type_name()
            ))),
        }
    }
}

macro_rules! register_function {
    ($lua:ident, $table:ident, $func_name:ident) => {
        $table
            .set(
                stringify!($func_name).to_case(Case::Pascal),
                $lua.create_function($func_name).unwrap(),
            )
            .unwrap();
    };
}

pub fn load_script_data_types(lua: &Lua, env: &Table) {
    register_function!(lua, env, point);
    register_function!(lua, env, size);
    register_function!(lua, env, rectangle);
    register_function!(lua, env, oled_image);
    register_function!(lua, env, bar);
    register_function!(lua, env, image);
    register_function!(lua, env, text);
    register_function!(lua, env, modifiers);
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

fn oled_image(_: &Lua, obj: OledImage) -> mlua::Result<OledImage> {
    Ok(obj)
}

fn bar(_: &Lua, obj: Bar) -> mlua::Result<Operation> {
    Ok(Operation::Bar(obj))
}

fn image(_: &Lua, obj: Image) -> mlua::Result<Operation> {
    Ok(Operation::Image(obj))
}

fn text(_: &Lua, obj: Text) -> mlua::Result<Operation> {
    Ok(Operation::Text(obj))
}

fn modifiers(_: &Lua, obj: Modifiers) -> mlua::Result<Modifiers> {
    Ok(obj)
}
