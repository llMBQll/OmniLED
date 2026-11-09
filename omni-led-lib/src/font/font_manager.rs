use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use freetype::Library;
use mlua::{ErrorContext, Lua, UserData, UserDataMethods};
use omni_led_derive::LuaName;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use crate::common::user_data::set_unique_user_data;
use crate::font::font::Font;
use crate::font::font_selector::FontSelector;
use crate::script_handler::script_data_types::FontWrapper;

#[derive(LuaName)]
pub struct FontManager {
    library: Library,
    fonts: HashMap<FontSelector, Rc<RefCell<Font>>>,
}

impl FontManager {
    pub fn load(lua: &Lua) {
        let manager = Self {
            library: Library::init().unwrap(),
            fonts: HashMap::new(),
        };
        set_unique_user_data(lua, manager);
    }

    pub fn get_font(&mut self, selector: FontSelector) -> mlua::Result<Rc<RefCell<Font>>> {
        if let Some(font) = self.fonts.get(&selector) {
            return Ok(font.clone());
        }

        let font = self.load_font(selector.clone())?;
        let font = Rc::new(RefCell::new(font));
        self.fonts.insert(selector, font.clone());
        Ok(font)
    }

    fn load_font(&self, selector: FontSelector) -> mlua::Result<Font> {
        let (font, font_index) = Self::load_font_data(selector.clone())
            .with_context(|_| format!("Loading font with selector: {:?}", selector))?;
        let font_bytes = font.copy_font_data().unwrap();

        let face = self
            .library
            .new_memory_face(font_bytes.to_vec(), font_index as isize)
            .map_err(mlua::Error::runtime)
            .with_context(|_| format!("Loading font with selector: {:?}", selector))?;

        Ok(Font::new(face))
    }

    fn load_font_data(selector: FontSelector) -> mlua::Result<(font_kit::font::Font, u32)> {
        match selector {
            FontSelector::Builtin => Ok(Self::load_builtin_font_data()),
            FontSelector::Filesystem(selector) => {
                let font_index = selector.font_index;
                let font = font_kit::font::Font::from_path(&selector.path, font_index)
                    .map_err(mlua::Error::external)?;
                Ok((font, font_index))
            }
            FontSelector::System(selector) => {
                let names: Vec<_> = selector.names.into_iter().map(|name| name.into()).collect();
                let properties = Properties {
                    style: selector.style.into(),
                    weight: selector.weight.into(),
                    stretch: selector.stretch.into(),
                };
                let handle = SystemSource::new()
                    .select_best_match(&names, &properties)
                    .map_err(mlua::Error::external)?;
                let font = handle.load().map_err(mlua::Error::external)?;
                Ok((font, 0 /* will this always be zero? */))
            }
        }
    }

    fn load_builtin_font_data() -> (font_kit::font::Font, u32) {
        const BUILTIN_FONT: &[u8] =
            include_bytes!("../../../assets/fonts/DejaVu/DejaVuSansMono.ttf");
        const BUILTIN_FONT_INDEX: u32 = 0;

        let builtin_font = Arc::new(BUILTIN_FONT.to_vec());
        let font = font_kit::font::Font::from_bytes(builtin_font, BUILTIN_FONT_INDEX).unwrap();

        (font, BUILTIN_FONT_INDEX)
    }
}

impl UserData for FontManager {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("get_font", |_, this, selector: FontSelector| {
            let font = this.get_font(selector)?;
            Ok(FontWrapper(font))
        });
    }
}
