use std::fmt::format;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use mlua::{Lua, Nil, Table, TableExt, UserData, UserDataFields, UserDataMethods};

pub const ON_SETTINGS_CHANGED: &str = "on_settings_changed";

pub struct Settings {
    update_interval: Duration,
    applications_file: String,
}

impl Settings {
    pub fn load(lua: &Lua) {
        static SETTINGS_SRC: &str = include_str!("settings.lua");

        lua.load(format!("Event:new('{}')", ON_SETTINGS_CHANGED).as_str()).exec().unwrap();

        let settings = Settings {
            update_interval: Duration::from_millis(100),
            applications_file: String::from("applications.lua"),
        };
        lua.globals().set("SETTINGS",settings).unwrap();

        lua.load(SETTINGS_SRC).exec().unwrap();
    }
}

impl UserData for Settings {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("update_interval", |_, this| {
            Ok(this.update_interval.as_millis())
        });
        fields.add_field_method_set("update_interval", |lua, this, val: u64| {
            this.update_interval = Duration::from_millis(val);
            let on_settings_changed: Table = lua.load(format!("EVENTS['{}']", ON_SETTINGS_CHANGED).as_str()).eval().unwrap();
            on_settings_changed.call_method("emit", ("update_interval", val))?;
            Ok(())
        });

        fields.add_field_method_get("applications_file", |_, this| {
            Ok(this.applications_file.clone())
        });
        fields.add_field_method_set("applications_file", |lua, this, val: String| {
            this.applications_file = val.clone();
            let on_settings_changed: Table = lua.load(format!("EVENTS['{}']", ON_SETTINGS_CHANGED).as_str()).eval().unwrap();
            on_settings_changed.call_method("emit", ("applications_file", val))?;
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