use mlua::{ErrorContext, FromLua, UserData};
use oled_derive::FromLuaTable;

use crate::model::rectangle::Rectangle;

#[derive(Clone, Debug, FromLua)]
pub enum Operation {
    Bar(Bar),
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
