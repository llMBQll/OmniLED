use mlua::UserData;
use crate::model::rectangle::Rectangle;

#[derive(Clone, Debug)]
pub enum Operation {
    Bar(Bar),
    Text(Text),
}

impl UserData for Operation {}

#[derive(Clone, Debug)]
pub struct Bar {
    pub value: f32,
    pub position: Rectangle,
    pub modifiers: Modifiers,
}

impl UserData for Bar {}

#[derive(Clone, Debug)]
pub struct Text {
    pub text: String,
    pub position: Rectangle,
    pub modifiers: Modifiers,
}

impl UserData for Text {}

#[derive(Clone, Copy, Debug, Default)]
pub struct Modifiers {
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
    pub strict: bool,
    pub upper: bool,
    pub vertical: bool,
    pub scrolling: bool,
}

impl UserData for Modifiers {}