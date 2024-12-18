use mlua::{ErrorContext, FromLua};
use omni_led_derive::FromLuaValue;

#[derive(Debug, Clone, FromLuaValue)]
pub enum FontSelector {
    Default,
    Filesystem(FilesystemSelector),
    System(SystemSelector),
}

#[derive(Debug, Clone, FromLuaValue)]
pub struct FilesystemSelector {
    pub path: String,
    #[mlua(default(0))]
    pub font_index: u32,
}

#[derive(Debug, Clone, FromLuaValue)]
pub struct SystemSelector {
    pub names: Vec<FamilyName>,
    #[mlua(default(Style::Normal))]
    pub style: Style,
    #[mlua(default(Weight::Normal))]
    pub weight: Weight,
    #[mlua(default(Stretch::Normal))]
    pub stretch: Stretch,
}

#[derive(Debug, Clone)]
pub struct FamilyName {
    pub title: String,
}

impl FromLua for FamilyName {
    fn from_lua(value: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::String(string) => {
                let title = string.to_string_lossy();
                Ok(Self { title })
            }
            mlua::Value::UserData(user_data) => {
                let data = user_data.borrow::<FamilyName>()?;
                Ok(data.clone())
            }
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: String::from("FamilyName"),
                message: None,
            }),
        }
    }
}

impl Into<font_kit::family_name::FamilyName> for FamilyName {
    fn into(self) -> font_kit::family_name::FamilyName {
        match self.title.as_str() {
            "Serif" => font_kit::family_name::FamilyName::Serif,
            "SansSerif" => font_kit::family_name::FamilyName::SansSerif,
            "Monospace" => font_kit::family_name::FamilyName::Monospace,
            "Cursive" => font_kit::family_name::FamilyName::Cursive,
            "Fantasy" => font_kit::family_name::FamilyName::Fantasy,
            _ => font_kit::family_name::FamilyName::Title(self.title),
        }
    }
}

#[derive(Debug, Clone, FromLuaValue)]
pub enum Style {
    Normal,
    Italic,
    Oblique,
}

impl Into<font_kit::properties::Style> for Style {
    fn into(self) -> font_kit::properties::Style {
        match self {
            Style::Normal => font_kit::properties::Style::Normal,
            Style::Italic => font_kit::properties::Style::Italic,
            Style::Oblique => font_kit::properties::Style::Oblique,
        }
    }
}

#[derive(Debug, Clone, FromLuaValue)]
pub enum Weight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

impl Into<font_kit::properties::Weight> for Weight {
    fn into(self) -> font_kit::properties::Weight {
        match self {
            Weight::Thin => font_kit::properties::Weight::THIN,
            Weight::ExtraLight => font_kit::properties::Weight::EXTRA_LIGHT,
            Weight::Light => font_kit::properties::Weight::LIGHT,
            Weight::Normal => font_kit::properties::Weight::NORMAL,
            Weight::Medium => font_kit::properties::Weight::MEDIUM,
            Weight::SemiBold => font_kit::properties::Weight::SEMIBOLD,
            Weight::Bold => font_kit::properties::Weight::BOLD,
            Weight::ExtraBold => font_kit::properties::Weight::EXTRA_BOLD,
            Weight::Black => font_kit::properties::Weight::BLACK,
        }
    }
}

#[derive(Debug, Clone, FromLuaValue)]
pub enum Stretch {
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    Normal,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

impl Into<font_kit::properties::Stretch> for Stretch {
    fn into(self) -> font_kit::properties::Stretch {
        match self {
            Stretch::UltraCondensed => font_kit::properties::Stretch::ULTRA_CONDENSED,
            Stretch::ExtraCondensed => font_kit::properties::Stretch::EXTRA_CONDENSED,
            Stretch::Condensed => font_kit::properties::Stretch::CONDENSED,
            Stretch::SemiCondensed => font_kit::properties::Stretch::SEMI_CONDENSED,
            Stretch::Normal => font_kit::properties::Stretch::NORMAL,
            Stretch::SemiExpanded => font_kit::properties::Stretch::SEMI_EXPANDED,
            Stretch::Expanded => font_kit::properties::Stretch::EXPANDED,
            Stretch::ExtraExpanded => font_kit::properties::Stretch::EXTRA_EXPANDED,
            Stretch::UltraExpanded => font_kit::properties::Stretch::ULTRA_EXPANDED,
        }
    }
}
