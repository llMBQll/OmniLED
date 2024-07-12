use convert_case::{Case, Casing};
use mlua::{ErrorContext, FromLua, Lua, Table, UserData, UserDataFields};
use oled_derive::FromLuaValue;

#[derive(Debug, Clone, Copy, FromLuaValue)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl UserData for Point {}

#[derive(Debug, Clone, Copy, FromLuaValue)]
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

#[derive(Debug, Clone, Copy, FromLuaValue)]
pub struct Rectangle {
    pub origin: Point,
    pub size: Size,
}

impl UserData for Rectangle {}

#[derive(Debug, Clone, FromLuaValue)]
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

#[derive(Clone, Debug, FromLuaValue)]
pub struct Bar {
    pub value: f32,
    pub position: Rectangle,

    #[mlua(default)]
    pub modifiers: Modifiers,
}

impl UserData for Bar {}

#[derive(Clone, Debug, FromLuaValue)]
pub struct Image {
    pub image: OledImage,
    pub position: Rectangle,

    #[mlua(default)]
    pub modifiers: Modifiers,
}

impl UserData for Image {}

#[derive(Clone, Debug, FromLuaValue)]
pub struct Text {
    pub text: String,
    pub position: Rectangle,

    #[mlua(default)]
    pub modifiers: Modifiers,
}

impl UserData for Text {}

#[derive(Clone, Copy, Debug, Default, FromLuaValue)]
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

#[derive(Clone, Copy, FromLuaValue)]
pub enum MemoryRepresentation {
    BitPerPixel,
    BytePerPixel,
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
