use log::debug;
use mlua::{IntoLua, Lua, UserData, UserDataFields, chunk};
use std::time::Duration;

use crate::common::lua_traits::LuaName;
use crate::common::user_data::{UserDataRef, set_unique_user_data};
use crate::constants::config::{ConfigType, load_config};
use crate::create_table_with_defaults;
use crate::events::event_queue::{Event, EventQueue};
use crate::events::events::ScriptEvent;
use crate::logging::logger::{LevelFilter, Log};
use crate::renderer::font_selector::FontSelector;
use crate::script_handler::script_data_types::DurationWrapper;

#[derive(Debug, Clone)]
pub struct Settings {
    pub animation_ticks_delay: usize,
    pub animation_ticks_rate: usize,
    pub font: FontSelector,
    pub log_level: LevelFilter,
    pub keyboard_ticks_repeat_delay: usize,
    pub keyboard_ticks_repeat_rate: usize,
    pub update_interval: Duration,
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
        logger.get().set_level_filter(settings.get().log_level);

        debug!("Loaded settings {:?}", settings.get());
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            animation_ticks_delay: 8,
            animation_ticks_rate: 2,
            font: FontSelector::Default,
            log_level: LevelFilter::Info,
            keyboard_ticks_repeat_delay: 2,
            keyboard_ticks_repeat_rate: 2,
            update_interval: Duration::from_millis(100),
        }
    }
}

impl LuaName for Settings {
    const NAME: &str = "Settings";
}

macro_rules! get_set_impl {
    ($fields:ident, $name:ident) => {
        $fields.add_field_method_get(stringify!($name), |_, this| Ok(this.$name.clone()));
        $fields.add_field_method_set(stringify!($name), |lua, this, val| {
            println!("Settings.{} = {:?}", stringify!($name), val);
            this.$name = val;
            EventQueue::instance()
                .lock()
                .unwrap()
                .push(Event::Script(ScriptEvent {
                    event: format!("Settings.{}", stringify!($name)),
                    value: this.$name.clone().into_lua(&lua)?,
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
        fields.add_field_method_get("update_interval", |_, this| {
            Ok(DurationWrapper(this.update_interval))
        });
        fields.add_field_method_set("update_interval", |lua, this, val: DurationWrapper| {
            this.update_interval = val.0;
            EventQueue::instance()
                .lock()
                .unwrap()
                .push(Event::Script(ScriptEvent {
                    event: "Settings.update_interval".to_string(),
                    value: val.into_lua(&lua)?,
                }));
            Ok(())
        });
    }
}
