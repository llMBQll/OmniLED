use mlua::{ErrorContext, FromLua, Lua, Table, UserData, UserDataFields, Value};
use omni_led_derive::FromLuaValue;
use std::hash::Hash;

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
pub struct ImageData {
    #[mlua(transform = Self::parse_format)]
    pub format: image::ImageFormat,
    pub bytes: Vec<u8>,
    pub hash: Option<u64>,
}

/// This is a private enum used just to facilitate easier parsing into image::ImageFormat type
#[derive(Clone, Debug, FromLuaValue)]
enum ImageFormatEnum {
    Avif,
    Bmp,
    Dds,
    Farbfeld,
    Gif,
    Hdr,
    Ico,
    Jpeg,
    OpenExr,
    Pcx,
    Png,
    Pnm,
    Qoi,
    Tga,
    Tiff,
    WebP,
}

impl UserData for ImageFormatEnum {}

impl ImageData {
    fn parse_format(format: ImageFormatEnum, _: &Lua) -> mlua::Result<image::ImageFormat> {
        match format {
            ImageFormatEnum::Avif => Ok(image::ImageFormat::Avif),
            ImageFormatEnum::Bmp => Ok(image::ImageFormat::Bmp),
            ImageFormatEnum::Dds => Ok(image::ImageFormat::Dds),
            ImageFormatEnum::Farbfeld => Ok(image::ImageFormat::Farbfeld),
            ImageFormatEnum::Gif => Ok(image::ImageFormat::Gif),
            ImageFormatEnum::Hdr => Ok(image::ImageFormat::Hdr),
            ImageFormatEnum::Ico => Ok(image::ImageFormat::Ico),
            ImageFormatEnum::Jpeg => Ok(image::ImageFormat::Jpeg),
            ImageFormatEnum::OpenExr => Ok(image::ImageFormat::OpenExr),
            ImageFormatEnum::Pcx => Ok(image::ImageFormat::Pcx),
            ImageFormatEnum::Png => Ok(image::ImageFormat::Png),
            ImageFormatEnum::Pnm => Ok(image::ImageFormat::Pnm),
            ImageFormatEnum::Qoi => Ok(image::ImageFormat::Qoi),
            ImageFormatEnum::Tga => Ok(image::ImageFormat::Tga),
            ImageFormatEnum::Tiff => Ok(image::ImageFormat::Tiff),
            ImageFormatEnum::WebP => Ok(image::ImageFormat::WebP),
        }
    }
}

impl UserData for ImageData {}

#[derive(Clone, Debug, FromLua)]
pub enum Widget {
    Bar(Bar),
    Image(Image),
    Text(Text),
}

impl UserData for Widget {}

#[derive(Clone, Debug, FromLuaValue)]
#[mlua(validate = Self::validate_range)]
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
    #[mlua(default = Range {min: 0.0, max: 100.0})]
    pub range: Range,
    #[mlua(default = false)]
    pub vertical: bool,
    pub position: Point,
    pub size: Size,

    #[mlua(default)]
    pub modifiers: Modifiers,
}

impl UserData for Bar {}

#[derive(FromLuaValue, Debug, PartialEq, Copy, Clone)]
pub enum Repeat {
    Once,
    ForDuration,
}

#[derive(Clone, Debug, FromLuaValue)]
pub struct Image {
    pub image: ImageData,
    #[mlua(default = false)]
    pub animated: bool,
    #[mlua(default = 128)]
    pub threshold: u8,
    #[mlua(default = Repeat::ForDuration)]
    pub repeats: Repeat,
    pub animation_group: Option<usize>,
    pub animation_ticks_delay: Option<usize>,
    pub animation_ticks_rate: Option<usize>,
    pub position: Point,
    pub size: Size,

    #[mlua(default)]
    pub modifiers: Modifiers,
}

impl UserData for Image {}

#[derive(Clone, Debug, FromLuaValue)]
pub struct Text {
    pub text: String,
    pub text_offset: Option<isize>,
    #[mlua(default = FontSize::Auto)]
    pub font_size: FontSize,
    #[mlua(default = false)]
    pub scrolling: bool,
    #[mlua(default = Repeat::ForDuration)]
    pub repeats: Repeat,
    pub animation_group: Option<usize>,
    pub animation_ticks_delay: Option<usize>,
    pub animation_ticks_rate: Option<usize>,
    pub position: Point,
    pub size: Size,
    pub hash: Option<u64>,

    #[mlua(default)]
    pub modifiers: Modifiers,
}

impl UserData for Text {}

#[derive(Copy, Clone, Debug)]
pub enum FontSize {
    Value(usize),
    Auto,
    AutoUpper,
}

impl UserData for FontSize {}

impl FromLua for FontSize {
    fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
        match value {
            Value::Integer(value) => Ok(FontSize::Value(value as usize)),
            Value::String(value) => {
                let value = value.to_string_lossy();
                match value.as_str() {
                    "Auto" => Ok(FontSize::Auto),
                    "AutoUpper" => Ok(FontSize::AutoUpper),
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
    #[mlua(default = false)]
    pub clear_background: bool,

    #[mlua(default = false)]
    pub flip_horizontal: bool,

    #[mlua(default = false)]
    pub flip_vertical: bool,

    #[mlua(default = false)]
    pub negative: bool,
}

impl UserData for Modifiers {}

#[derive(Clone, Copy, FromLuaValue)]
pub enum MemoryLayout {
    #[mlua(alias = "SteelSeries")]
    BitPerPixel,
    BytePerPixel,
    #[mlua(alias = "SteelSeries2")]
    BitPerPixelVertical,
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
