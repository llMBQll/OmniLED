use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::model::position::Position;

#[derive(Serialize, Deserialize, Debug)]
pub struct Display {
    pub name: String,
    pub parts: Vec<(String, Position)>,
    pub sensitivity_list: HashSet<String>,
}

impl Display {
    pub fn new(name: String, parts: Vec<(String, Position)>, sensitivity_list: HashSet<String>) -> Self {
        Self {
            name,
            parts,
            sensitivity_list
        }
    }
}