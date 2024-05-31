use mlua::{Lua, Value};

use crate::screen::debug_output::debug_output_settings::DebugOutputSettings;
use crate::screen::screen::{Buffer, MemoryRepresentation, Screen, Settings, Size};

pub struct DebugOutput {
    name: String,
    screen_size: Size,
}

impl Screen for DebugOutput {
    fn init(lua: &Lua, settings: Value) -> mlua::Result<Self> {
        let settings = DebugOutputSettings::new(lua, settings)?;

        Ok(Self {
            name: settings.name,
            screen_size: settings.screen_size,
        })
    }

    fn size(&mut self, _lua: &Lua) -> mlua::Result<Size> {
        Ok(self.screen_size)
    }

    fn update(&mut self, _lua: &Lua, buffer: Buffer) -> mlua::Result<()> {
        for _ in 0..self.screen_size.width {
            print!("-");
        }
        println!();

        for row in buffer.rows() {
            for pixel in row {
                if *pixel == 0 {
                    print!(" ");
                } else {
                    print!("0");
                }
            }
            println!();
        }

        Ok(())
    }

    fn name(&mut self, _lua: &Lua) -> mlua::Result<String> {
        Ok(self.name.clone())
    }

    fn memory_representation(&mut self, _lua: &Lua) -> mlua::Result<MemoryRepresentation> {
        Ok(MemoryRepresentation::BytePerPixel)
    }
}
