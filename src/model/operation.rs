use mlua::UserData;
use crate::model::rectangle::Rectangle;

#[derive(Clone, Debug)]
pub enum Operation {
    Bar(Bar),
    Text(Text),
    ScrollingText(ScrollingText),
}

impl UserData for Operation {}

#[derive(Clone, Debug)]
pub struct Bar {
    pub value: f32,
    pub modifiers: Modifiers,
    pub position: Rectangle,
}

impl UserData for Bar {}

#[derive(Clone, Debug)]
pub struct Text {
    pub text: String,
    pub modifiers: Modifiers,
    pub position: Rectangle,
}

impl UserData for Text {}

#[derive(Clone, Debug)]
pub struct ScrollingText {
    pub text: String,
    pub count: i32,
    pub modifiers: Modifiers,
    pub position: Rectangle,
}

impl UserData for ScrollingText {}

#[derive(Clone, Copy, Debug, Default)]
pub struct Modifiers {
    pub inverted: bool,
    pub reverse: bool,
    pub strict: bool,
    pub upper: bool,
    pub vertical: bool,
}

impl UserData for Modifiers {}