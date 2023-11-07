use mlua::{FromLua, UserData};

use crate::model::rectangle::Rectangle;

#[derive(Clone, Debug, FromLua)]
pub enum Operation {
    Bar(Bar),
    Text(Text),
}

impl UserData for Operation {}

#[derive(Clone, Debug, FromLua)]
pub struct Bar {
    pub value: f32,
    pub position: Rectangle,
    pub modifiers: Modifiers,
}

impl UserData for Bar {}

#[derive(Clone, Debug, FromLua)]
pub struct Text {
    pub text: String,
    pub position: Rectangle,
    pub modifiers: Modifiers,
}

impl UserData for Text {}

#[derive(Clone, Copy, Debug, Default, FromLua)]
pub struct Modifiers {
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
    pub strict: bool,
    pub vertical: bool,
    pub scrolling: bool,
    pub font_size: Option<usize>,
}

impl UserData for Modifiers {}
