use mlua::{Lua, Value};

use crate::screen::debug_output::debug_output_settings::DebugOutputSettings;
use crate::screen::screen::{Result, Screen, Settings, Size};

pub struct DebugOutput {
    name: String,
    size: Size,
}

impl Screen for DebugOutput {
    fn init(lua: &Lua, settings: Value) -> Result<Self> {
        let settings = DebugOutputSettings::new(lua, settings).unwrap();

        Ok(Self {
            name: settings.name,
            size: settings.size,
        })
    }

    fn size(&mut self, _lua: &Lua) -> Result<Size> {
        Ok(self.size)
    }

    fn update(&mut self, _lua: &Lua, pixels: Vec<u8>) -> Result<()> {
        for _ in 0..self.size.width {
            print!("-");
        }
        println!();

        for chunk in pixels.chunks(self.size.width / 8) {
            for byte in chunk {
                for i in 0..8 {
                    if (byte >> (7 - i) & 0b00000001) == 1 {
                        print!("0");
                    } else {
                        print!(" ");
                    }
                }
            }
            println!();
        }

        Ok(())
    }

    fn name(&mut self, _lua: &Lua) -> Result<String> {
        Ok(self.name.clone())
    }
}
