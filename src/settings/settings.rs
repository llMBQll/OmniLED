use std::time::Duration;
use mlua::{FromLua, Function, Lua, Table, ToLua, UserData, UserDataFields, UserDataMethods};


pub struct Settings<'a> {
    _lua: &'a Lua,
}

impl<'a> Settings<'a> {
    pub fn new(lua: &'a Lua) -> Self {
        static SETTINGS_SRC: &str = include_str!("settings.lua");

        lua.globals().set("SETTINGS", SettingsData::new()).unwrap();
        lua.globals().set("DEFAULT_SETTINGS", SettingsData::new()).unwrap();
        lua.load(SETTINGS_SRC).exec().unwrap();

        Self {
            _lua: lua,
        }
    }
}

struct SettingsData {
    update_interval: Duration,
    applications_file: String,
}

impl SettingsData {
    fn new() -> Self {
        Self {
            update_interval: Duration::from_millis(100),
            applications_file: String::from("applications.lus"),
        }
    }
}

impl UserData for SettingsData {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("update_interval", |_, this| {
            Ok(this.update_interval.as_millis())
        });
        fields.add_field_method_set("update_interval", |_, this, val: u64| {
            this.update_interval = Duration::from_millis(val);
            Ok(())
        });

        fields.add_field_method_get("applications_file", |_, this| {
            Ok(this.applications_file.clone())
        });
        fields.add_field_method_set("applications_file", |_, this, val: String| {
            this.applications_file = val;
            Ok(())
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function::<_, String, _, _>("key_exists", |_, key| {
            let res = match key.as_str() {
                "update_interval" => true,
                "applications_file" => true,
                _ => false,
            };
            Ok(res)
        });
    }
}