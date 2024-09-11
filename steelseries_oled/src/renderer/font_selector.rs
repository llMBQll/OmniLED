use mlua::{ErrorContext, FromLua};
use oled_derive::FromLuaValue;

#[derive(Debug, Clone, FromLuaValue)]
pub enum FontSelector {
    Default,
    Filesystem(FilesystemSelector),
    System(SystemSelector),
}

#[derive(Debug, Clone, FromLuaValue)]
pub struct FilesystemSelector {
    pub path: String,
    pub font_index: Option<u32>,
}

#[derive(Debug, Clone, FromLuaValue)]
pub struct SystemSelector {
    pub names: Vec<FamilyName>,
    pub style: Style,
    pub weight: Weight,
    pub stretch: Stretch,
}

#[derive(Debug, Clone, FromLuaValue)]
pub enum FamilyName {
    Title(String),
    Serif,
    SansSerif,
    Monospace,
    Cursive,
    Fantasy,
}

impl Into<font_kit::family_name::FamilyName> for FamilyName {
    fn into(self) -> font_kit::family_name::FamilyName {
        match self {
            FamilyName::Title(title) => font_kit::family_name::FamilyName::Title(title),
            FamilyName::Serif => font_kit::family_name::FamilyName::Serif,
            FamilyName::SansSerif => font_kit::family_name::FamilyName::SansSerif,
            FamilyName::Monospace => font_kit::family_name::FamilyName::Monospace,
            FamilyName::Cursive => font_kit::family_name::FamilyName::Cursive,
            FamilyName::Fantasy => font_kit::family_name::FamilyName::Fantasy,
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
