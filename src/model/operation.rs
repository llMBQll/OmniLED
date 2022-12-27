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
    pub modifiers: Modifiers,
    pub position: Position,
}

#[derive(Debug)]
pub struct Text {
    pub text: String,
    pub modifiers: Modifiers,
    pub position: Position,
}

#[derive(Debug)]
pub struct ScrollingText {
    pub text: String,
    pub count: i32,
    pub modifiers: Modifiers,
    pub position: Position,
}

#[derive(Debug, Default)]
pub struct Modifiers {
    pub inverted: bool,
    pub reverse: bool,
    pub strict: bool,
    pub upper: bool,
    pub vertical: bool,
}