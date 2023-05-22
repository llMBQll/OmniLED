use crate::screen::screen::{Screen, Size};
use crate::screen::screen::Result;
use crate::screen::steelseries_engine::api;

pub struct SteelseriesEngine {
    size: Size,
}

impl SteelseriesEngine {
    pub fn new() -> Self {
        Self {
            size: Size { width: 128, height: 40 },
        }
    }
}

impl Screen for SteelseriesEngine {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn size(&mut self) -> Result<Size> {
        Ok(self.size)
    }

    fn update(&mut self, pixels: &Vec<u8>) -> Result<()> {
        api::update(pixels);
        Ok(())
    }

    fn name(&self) -> Result<String> {
        Ok(String::from("Steelseries Engine"))
    }
}