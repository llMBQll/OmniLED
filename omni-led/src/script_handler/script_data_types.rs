use mlua::{ErrorContext, FromLua, Lua, Table, UserData, UserDataFields, Value};
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
pub enum Widget {
    Bar(Bar),
    Image(Image),
    Text(Text),
}

impl UserData for Widget {}

#[derive(Clone, Debug, FromLuaValue)]
#[mlua(validate(Self::validate_range))]
pub struct Range {
    pub min: f32,
    pub max: f32,
}

impl Range {
    fn validate_range(range: &Self) -> mlua::Result<()> {
        if range.min <= range.max {
            Ok(())
        } else {
            Err(mlua::Error::runtime(format!(
                "range.max shall be greater or equal than range.min, got min = {}, max = {}",
                range.min, range.max
            )))
        }
    }
}

impl UserData for Range {}

#[derive(Clone, Debug, FromLuaValue)]
pub struct Bar {
    pub value: f32,
    #[mlua(default(Range {min: 0.0, max: 100.0}))]
    pub range: Range,
    #[mlua(default(false))]
    pub vertical: bool,
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
    #[mlua(default(Offset::Auto))]
    pub text_offset: Offset,
    pub font_size: Option<usize>,
    #[mlua(default(false))]
    pub scrolling: bool,
    pub position: Point,
    pub size: Size,

    #[mlua(default)]
    pub modifiers: Modifiers,
}

impl UserData for Text {}

#[derive(Copy, Clone, Debug)]
pub enum Offset {
    Value(isize),
    Auto,
    AutoUpper,
}

impl UserData for Offset {}

impl FromLua for Offset {
    fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
        match value {
            Value::Integer(value) => Ok(Offset::Value(value as isize)),
            Value::String(value) => {
                let value = value.to_string_lossy();
                match value.as_str() {
                    "Auto" => Ok(Offset::Auto),
                    "AutoUpper" => Ok(Offset::AutoUpper),
                    _ => Err(mlua::Error::runtime(format!(
                        "Expected one of ['Auto', 'AutoUpper'], got '{}'",
                        value
                    ))),
                }
            }
            other => Err(mlua::Error::runtime(format!(
                "Expected type 'integer' or 'string', got '{}'",
                other.type_name()
            ))),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, FromLuaValue)]
pub struct Modifiers {
    #[mlua(default(false))]
    pub clear_background: bool,

    #[mlua(default(false))]
    pub flip_horizontal: bool,

    #[mlua(default(false))]
    pub flip_vertical: bool,

    #[mlua(default(false))]
    pub negative: bool,
}

impl UserData for Modifiers {}

#[derive(Clone, Copy, FromLuaValue)]
pub enum MemoryRepresentation {
    BitPerPixel,
    BytePerPixel,
}

pub fn load_script_data_types(lua: &Lua, env: &Table) {
    macro_rules! register_widget {
        ($lua:ident, $table:ident, $type_name:ident) => {
            $table
                .set(
                    stringify!($type_name),
                    $lua.create_function(|_: &Lua, obj: $type_name| Ok(Widget::$type_name(obj)))
                        .unwrap(),
                )
                .unwrap();
        };
    }

    register_widget!(lua, env, Bar);
    register_widget!(lua, env, Image);
    register_widget!(lua, env, Text);
}
