use crate::model::position::Position;

#[derive(Debug)]
pub enum Operation {
    Bar(Bar),
    Text(Text),
    ScrollingText(ScrollingText),
}

#[derive(Debug)]
pub struct Bar {
    pub value: f32,
    pub position: Position,
}

#[derive(Debug)]
pub struct Text {
    pub text: String,
    pub strict: bool,
    pub upper: bool,
    pub position: Position,
}

#[derive(Debug)]
pub struct ScrollingText {
    pub text: String,
    pub strict: bool,
    pub upper: bool,
    pub count: i32,
    pub position: Position,
}