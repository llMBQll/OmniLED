#[derive(Copy)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

#[derive(Copy)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

#[derive(Copy)]
pub struct Rectangle {
    pub origin: Point,
    pub size: Size,
}