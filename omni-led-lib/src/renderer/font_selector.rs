use mlua::{ErrorContext, FromLua, Lua, Table, UserData};
use omni_led_derive::{FromLuaValue, LuaEnum};

pub fn set_font_selector_enums(lua: &Lua, env: &Table) {
    FontSelector::set_lua_enum(lua, env).unwrap();
    FamilyName::set_lua_enum(lua, env).unwrap();
    Style::set_lua_enum(lua, env).unwrap();
    Weight::set_lua_enum(lua, env).unwrap();
    Stretch::set_lua_enum(lua, env).unwrap();
}

#[derive(Debug, Clone, PartialEq, LuaEnum)]
pub enum FontSelector {
    Default,
    Filesystem(FilesystemSelector),
    System(SystemSelector),
}

impl UserData for FontSelector {}

#[derive(Debug, Clone, PartialEq, FromLuaValue)]
pub struct FilesystemSelector {
    pub path: String,
    #[mlua(default = 0)]
    pub font_index: u32,
}

#[derive(Debug, Clone, PartialEq, FromLuaValue)]
pub struct SystemSelector {
    pub names: Vec<FamilyName>,
    #[mlua(default = Style::Normal)]
    pub style: Style,
    #[mlua(default = Weight::Normal)]
    pub weight: Weight,
    #[mlua(default = Stretch::Normal)]
    pub stretch: Stretch,
}

#[derive(Debug, Clone, PartialEq, LuaEnum)]
pub enum FamilyName {
    Title(String),
    Serif,
    SansSerif,
    Monospace,
    Cursive,
    Fantasy,
}

impl UserData for FamilyName {}

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

#[derive(Debug, Clone, PartialEq, LuaEnum)]
pub enum Style {
    Normal,
    Italic,
    Oblique,
}

impl UserData for Style {}

impl Into<font_kit::properties::Style> for Style {
    fn into(self) -> font_kit::properties::Style {
        match self {
            Style::Normal => font_kit::properties::Style::Normal,
            Style::Italic => font_kit::properties::Style::Italic,
            Style::Oblique => font_kit::properties::Style::Oblique,
        }
    }
}

#[derive(Debug, Clone, PartialEq, LuaEnum)]
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

impl UserData for Weight {}

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

#[derive(Debug, Clone, PartialEq, LuaEnum)]
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

impl UserData for Stretch {}

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

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::chunk;

    macro_rules! eval {
        ($lua:ident, $code:tt) => {
            $lua.load(chunk! $code)
            .eval::<FontSelector>()
            .unwrap()
        };
    }

    fn get_lua_env() -> Lua {
        let lua = Lua::new();
        set_font_selector_enums(&lua, &lua.globals());
        lua
    }

    #[test]
    fn default_selector() {
        let lua = get_lua_env();
        let selector = eval!(lua, { FontSelector.Default });
        assert_eq!(selector, FontSelector::Default);
    }

    #[test]
    fn filesystem_selector() {
        const PATH: &str = "my/path";
        const INDEX: u32 = 7;

        let lua = get_lua_env();
        let selector =
            eval!(lua, { FontSelector.Filesystem({ path = $PATH, font_index = $INDEX }) });
        assert_eq!(
            selector,
            FontSelector::Filesystem(FilesystemSelector {
                path: PATH.to_string(),
                font_index: INDEX,
            })
        );
    }

    #[test]
    fn system_selector() {
        const TITLE: &str = "font-title";

        let lua = get_lua_env();
        let selector = eval!(lua, { FontSelector.System({
            names = { FamilyName.Title($TITLE), FamilyName.Monospace },
            style = Style.Italic,
            weight = Weight.Thin,
            stretch = Stretch.SemiCondensed,
        }) });

        assert_eq!(
            selector,
            FontSelector::System(SystemSelector {
                names: vec![FamilyName::Title(TITLE.to_string()), FamilyName::Monospace],
                style: Style::Italic,
                weight: Weight::Thin,
                stretch: Stretch::SemiCondensed,
            })
        );
    }
}
