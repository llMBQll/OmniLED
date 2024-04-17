use mlua::{Lua, Value};

use crate::screen::debug_output::debug_output_settings::DebugOutputSettings;
use crate::screen::screen::{Screen, Settings, Size};

pub struct DebugOutput {
    name: String,
    size: Size,
}

impl Screen for DebugOutput {
    fn init(lua: &Lua, settings: Value) -> mlua::Result<Self> {
        let settings = DebugOutputSettings::new(lua, settings)?;

        Ok(Self {
            name: settings.name,
            size: settings.size,
        })
    }

    fn size(&mut self, _lua: &Lua) -> mlua::Result<Size> {
        Ok(self.size)
    }

    fn update(&mut self, _lua: &Lua, pixels: Vec<u8>) -> mlua::Result<()> {
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

    fn name(&mut self, _lua: &Lua) -> mlua::Result<String> {
        Ok(self.name.clone())
    }
}
