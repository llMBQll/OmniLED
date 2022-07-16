use std::collections::HashMap;

use serde_json::Value;
use subprocess::{Popen, PopenConfig, PopenError};


pub struct Plugin {
    process: Popen,
    name: String,
}

impl Plugin {
    pub fn new(path: &String) -> Result<Self, PopenError> {
        Ok(Self {
            process: Popen::create(&["runner", path.as_str()], PopenConfig::default())?,
            name: String::new()
        })
    }
}