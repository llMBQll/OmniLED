use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

// impl Position {
//     pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
//         Self {
//             x,
//             y,
//             width,
//             height,
//         }
//     }
// }