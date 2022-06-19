use crate::Position;

#[derive(Debug)]
pub enum Operation {
    Bar(Bar),
    Text(Text),
    FixedHeight(FixedHeight),
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
    pub position: Position,
}

#[derive(Debug)]
pub struct FixedHeight {
    pub text: String,
    pub position: Position,
}

#[derive(Debug)]
pub struct ScrollingText {
    pub text: String,
    pub count: i32,
    pub position: Position,
}