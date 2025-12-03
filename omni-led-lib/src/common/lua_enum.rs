use mlua::{Lua, Table};

use crate::{
    devices::device::MemoryLayout,
    logging::logger::LevelFilter,
    renderer::font_selector::{FamilyName, FontSelector, Stretch, Style, Weight},
    script_handler::script_data_types::{FontSize, ImageFormat, Repeat, Widget},
};

pub fn set_lua_enums(lua: &Lua, env: &Table) {
    FamilyName::set_lua_enum(lua, env).unwrap();
    FontSelector::set_lua_enum(lua, env).unwrap();
    FontSize::set_lua_enum(lua, env).unwrap();
    ImageFormat::set_lua_enum(lua, env).unwrap();
    LevelFilter::set_lua_enum(lua, env).unwrap();
    MemoryLayout::set_lua_enum(lua, env).unwrap();
    Repeat::set_lua_enum(lua, env).unwrap();
    Stretch::set_lua_enum(lua, env).unwrap();
    Style::set_lua_enum(lua, env).unwrap();
    Weight::set_lua_enum(lua, env).unwrap();
    Widget::set_lua_enum(lua, env).unwrap();
}
