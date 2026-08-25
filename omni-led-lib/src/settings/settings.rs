use log::debug;
use mlua::{IntoLua, Lua, UserData, UserDataFields, chunk};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use crate::common::lua_traits::LuaName;
use crate::common::user_data::{UserDataRef, set_unique_user_data};
use crate::constants::config::{ConfigType, load_config};
use crate::create_table_with_defaults;
use crate::events::event_queue::{Event, EventQueue};
use crate::events::events::Events;
use crate::events::events::ScriptEvent;
use crate::logging::logger::{LevelFilter, Log};
use crate::renderer::font_selector::FontSelector;
use crate::script_handler::script_data_types::DurationWrapper;
use crate::script_handler::script_data_types::EventKey;

#[derive(Debug, Clone)]
pub struct Settings {
    pub animation_ticks_delay: Rc<RefCell<usize>>,
    pub animation_ticks_rate: Rc<RefCell<usize>>,
    pub font: Rc<RefCell<FontSelector>>,
    pub log_level: Rc<RefCell<LevelFilter>>,
    pub keyboard_ticks_repeat_delay: Rc<RefCell<usize>>,
    pub keyboard_ticks_repeat_rate: Rc<RefCell<usize>>,
    pub update_interval: Rc<RefCell<DurationWrapper>>,
}

impl Settings {
    pub fn load(lua: &Lua, config: String) {
        set_unique_user_data(lua, Settings::default());

        let env = create_table_with_defaults!(lua, {
            Log = Log,
            PLATFORM = PLATFORM,
            Settings = Settings,
        });
        load_config(lua, ConfigType::Settings, &config, env).unwrap();

        let settings = UserDataRef::<Settings>::load(lua);
        let logger = UserDataRef::<Log>::load(lua);
        logger
            .get()
            .set_level_filter(*settings.get().log_level.borrow());

        debug!("Loaded settings {:?}", settings.get());
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            animation_ticks_delay: Rc::new(RefCell::new(8)),
            animation_ticks_rate: Rc::new(RefCell::new(2)),
            font: Rc::new(RefCell::new(FontSelector::Default)),
            log_level: Rc::new(RefCell::new(LevelFilter::Info)),
            keyboard_ticks_repeat_delay: Rc::new(RefCell::new(2)),
            keyboard_ticks_repeat_rate: Rc::new(RefCell::new(2)),
            update_interval: Rc::new(RefCell::new(DurationWrapper(Duration::from_millis(100)))),
        }
    }
}

macro_rules! make_register_fn {
    ($fn_name:ident, $name:ident, $ty:ty) => {
        pub fn $fn_name<F: Fn(&Lua, $ty) + 'static>(lua: &Lua, callback: F) {
            let key = EventKey::String(format!("Settings.{}", stringify!($name)));
            let callback = lua
                .create_function(move |lua, (_event, val): (String, $ty)| {
                    callback(lua, val);
                    Ok(())
                })
                .unwrap();
            Events::register(key, callback, true);
        }
    };
}

impl Settings {
    make_register_fn!(on_log_level_update, log_level, LevelFilter);
}

impl LuaName for Settings {
    const NAME: &str = "Settings";
}

macro_rules! get_set_impl {
    ($fields:ident, $name:ident) => {
        $fields.add_field_method_get(stringify!($name), |_, this| {
            Ok((*this.$name.borrow()).clone())
        });
        $fields.add_field_method_set(stringify!($name), |lua, this, val| {
            *this.$name.borrow_mut() = val;
            EventQueue::instance()
                .lock()
                .unwrap()
                .push(Event::Script(ScriptEvent {
                    event: format!("Settings.{}", stringify!($name)),
                    value: (*this.$name.borrow()).clone().into_lua(&lua)?,
                }));
            Ok(())
        });
    };
}

impl UserData for Settings {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        get_set_impl!(fields, animation_ticks_delay);
        get_set_impl!(fields, animation_ticks_rate);
        get_set_impl!(fields, font);
        get_set_impl!(fields, log_level);
        get_set_impl!(fields, keyboard_ticks_repeat_delay);
        get_set_impl!(fields, keyboard_ticks_repeat_rate);
        get_set_impl!(fields, update_interval);
    }
}
