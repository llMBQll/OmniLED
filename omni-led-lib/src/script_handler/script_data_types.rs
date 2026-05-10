use mlua::{
    ErrorContext, FromLua, Lua, MetaMethod, UserData, UserDataFields, UserDataMethods, Value,
};
use omni_led_derive::{FromLuaValue, LuaEnum};
use std::{hash::Hash, time::Duration};

use crate::common::lua_traits::{FromUserdata, LuaName, LuaTypeStaticMembers, StaticMembers};

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

impl ImageData {
    fn parse_format(format: ImageFormat, _: &Lua) -> mlua::Result<image::ImageFormat> {
        match format {
            ImageFormat::Avif => Ok(image::ImageFormat::Avif),
            ImageFormat::Bmp => Ok(image::ImageFormat::Bmp),
            ImageFormat::Dds => Ok(image::ImageFormat::Dds),
            ImageFormat::Farbfeld => Ok(image::ImageFormat::Farbfeld),
            ImageFormat::Gif => Ok(image::ImageFormat::Gif),
            ImageFormat::Hdr => Ok(image::ImageFormat::Hdr),
            ImageFormat::Ico => Ok(image::ImageFormat::Ico),
            ImageFormat::Jpeg => Ok(image::ImageFormat::Jpeg),
            ImageFormat::OpenExr => Ok(image::ImageFormat::OpenExr),
            #[allow(deprecated)]
            ImageFormat::Pcx => Ok(image::ImageFormat::Pcx),
            ImageFormat::Png => Ok(image::ImageFormat::Png),
            ImageFormat::Pnm => Ok(image::ImageFormat::Pnm),
            ImageFormat::Qoi => Ok(image::ImageFormat::Qoi),
            ImageFormat::Tga => Ok(image::ImageFormat::Tga),
            ImageFormat::Tiff => Ok(image::ImageFormat::Tiff),
            ImageFormat::WebP => Ok(image::ImageFormat::WebP),
        }
    }
}

impl UserData for ImageData {}

// 1:1 equivalent to image::ImageFormat, only used to facilitate the conversion from lua values
#[derive(Clone, Debug, LuaEnum)]
pub enum ImageFormat {
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

impl UserData for ImageFormat {}

#[derive(Clone, Debug, LuaEnum)]
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
                "range.max shall be greater than or equal to range.min, got min = {}, max = {}",
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

#[derive(Debug, PartialEq, Copy, Clone, LuaEnum)]
pub enum Repeat {
    Once,
    ForDuration,
}

impl UserData for Repeat {}

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

#[derive(Copy, Clone, Debug, LuaEnum)]
pub enum FontSize {
    #[mlua(implicit_construct)]
    Value(usize),
    Auto,
    AutoUpper,
}

impl UserData for FontSize {}

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

#[derive(Clone, Copy, LuaEnum)]
pub enum MemoryLayout {
    #[mlua(alias = "SteelSeries")]
    BitPerPixel,
    BytePerPixel,
    #[mlua(alias = "SteelSeries2")]
    BitPerPixelVertical,
}

impl UserData for MemoryLayout {}

#[derive(Clone)]
pub struct Regex {
    re: regex::Regex,
}

impl Regex {
    pub fn new(re: &str) -> mlua::Result<Self> {
        let re = regex::Regex::new(re).map_err(mlua::Error::external)?;
        Ok(Self { re })
    }

    pub fn matches(&self, string: &str) -> bool {
        self.re.is_match(string)
    }
}

impl LuaName for Regex {
    const NAME: &str = "Regex";
}

impl LuaTypeStaticMembers for Regex {
    fn add_members(functions: &mut StaticMembers<'_>) {
        functions.add_function("new", |_, re: String| Regex::new(&re))
    }
}

impl UserData for Regex {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("matches", |_, this, string: String| {
            Ok(this.matches(&string))
        });
    }
}

impl FromUserdata for Regex {}

impl FromLua for Regex {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        Self::from_userdata(lua, value)
    }
}

#[derive(Clone, LuaEnum)]
pub enum EventKey {
    #[mlua(implicit_construct)]
    Regex(Regex),
    #[mlua(implicit_construct)]
    String(String),
}

impl EventKey {
    pub fn matches(&self, event: &str) -> bool {
        match self {
            EventKey::Regex(regex) => regex.matches(event),
            EventKey::String(string) => string == event,
        }
    }
}

impl UserData for EventKey {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("matches", |_, this, string: String| {
            Ok(this.matches(&string))
        });
    }
}

#[derive(Clone)]
pub struct DurationWrapper(pub Duration);

impl DurationWrapper {
    pub fn transform(wrapper: Self, _: &Lua) -> mlua::Result<Duration> {
        Ok(wrapper.0)
    }
}

impl LuaName for DurationWrapper {
    const NAME: &str = "Duration";
}

impl LuaTypeStaticMembers for DurationWrapper {
    fn add_members(members: &mut StaticMembers<'_>) {
        macro_rules! construct {
            ($from:ident) => {
                members.add_function(stringify!($from), |_, value: u64| {
                    Ok(DurationWrapper(Duration::$from(value)))
                })
            };
        }

        construct!(from_nanos);
        construct!(from_micros);
        construct!(from_millis);
        construct!(from_secs);
        construct!(from_mins);
        construct!(from_hours);

        members.add_member("MAX", DurationWrapper(Duration::MAX));
        members.add_member("ZERO", DurationWrapper(Duration::ZERO));
    }
}

impl UserData for DurationWrapper {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        macro_rules! get {
            ($getter:ident) => {
                methods.add_method(stringify!($getter), |_, this, _: ()| Ok(this.0.$getter()))
            };
        }
        macro_rules! comparison {
            ($meta:expr, $op:tt) => {
                methods.add_meta_function(
                    $meta,
                    |_, (lhs, rhs): (Self, Self)| Ok(lhs.0 $op rhs.0),
                )
            };
        }

        get!(as_nanos);
        get!(as_micros);
        get!(as_millis);
        get!(as_secs);
        comparison!(MetaMethod::Lt, <);
        comparison!(MetaMethod::Le, <=);
        comparison!(MetaMethod::Eq, ==);
        methods.add_meta_function(MetaMethod::Add, |_, (lhs, rhs): (Self, Self)| {
            Ok(Self(lhs.0.saturating_add(rhs.0)))
        });
        methods.add_meta_function(MetaMethod::Sub, |_, (lhs, rhs): (Self, Self)| {
            Ok(Self(lhs.0.saturating_sub(rhs.0)))
        });
        methods.add_meta_method(MetaMethod::ToString, |_, this, _: ()| {
            Ok(humantime::format_duration(this.0).to_string())
        });
    }
}

impl FromUserdata for DurationWrapper {}

impl FromLua for DurationWrapper {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        Self::from_userdata(lua, value)
    }
}
