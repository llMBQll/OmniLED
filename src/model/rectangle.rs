use mlua::{UserData, UserDataFields};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl UserData for Point {}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

impl UserData for Size {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("width", |_, this| {
            Ok(this.width)
        });

        fields.add_field_method_get("height", |_, this| {
            Ok(this.height)
        });
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Rectangle {
    pub origin: Point,
    pub size: Size,
}

impl UserData for Rectangle {}