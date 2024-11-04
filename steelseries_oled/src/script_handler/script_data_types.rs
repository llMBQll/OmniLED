use mlua::{ErrorContext, FromLua, Lua, Table, UserData, UserDataFields};
use oled_derive::FromLuaValue;

#[derive(Debug, Clone, Copy, FromLuaValue)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl UserData for Point {}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, FromLuaValue)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

impl UserData for Size {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Width", |_, this| Ok(this.width));

        fields.add_field_method_get("Height", |_, this| Ok(this.height));
    }
}

#[derive(Debug, Clone, Copy, FromLuaValue)]
pub struct Rectangle {
    pub position: Point,
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
pub struct Range {
    pub min: f32,
    pub max: f32,
}

impl UserData for Range {}

#[derive(Clone, Debug, FromLuaValue)]
pub struct Bar {
    pub value: f32,
    #[mlua(default(Range {min: 0.0, max: 100.0}))]
    pub range: Range,
    pub position: Point,
    pub size: Size,

    #[mlua(default)]
    pub modifiers: Modifiers,
}

impl UserData for Bar {}

#[derive(Clone, Debug, FromLuaValue)]
pub struct Image {
    pub image: OledImage,
    pub position: Point,
    pub size: Size,

    #[mlua(default)]
    pub modifiers: Modifiers,
}

impl UserData for Image {}

#[derive(Clone, Debug, FromLuaValue)]
pub struct Text {
    pub text: String,
    pub position: Point,
    pub size: Size,

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

pub fn load_script_data_types(lua: &Lua, env: &Table) {
    macro_rules! register_function {
        ($lua:ident, $table:ident, $type_name:ident) => {
            $table
                .set(
                    stringify!($type_name),
                    $lua.create_function(|_: &Lua, obj: $type_name| Ok(Operation::$type_name(obj)))
                        .unwrap(),
                )
                .unwrap();
        };
    }

    register_function!(lua, env, Bar);
    register_function!(lua, env, Image);
    register_function!(lua, env, Text);
}
