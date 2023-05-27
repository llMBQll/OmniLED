use std::time::Duration;
use mlua::{Lua, UserData, UserDataFields, UserDataMethods};

pub struct Settings {
    update_interval: Duration,
    applications_file: String,
    application_timeout: Duration,
}

impl Settings {
    pub fn load(lua: &Lua) {
        static SETTINGS_SRC: &str = include_str!("settings.lua");

        let settings = Settings {
            update_interval: Duration::from_millis(100),
            applications_file: String::from("applications.lua"),
            application_timeout: Duration::from_millis(30000),
        };
        lua.globals().set("SETTINGS", settings).unwrap();

        lua.load(SETTINGS_SRC).exec().unwrap();
    }
}

impl UserData for Settings {
    // TODO use a macro for the repetitive code

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

        fields.add_field_method_get("application_timeout", |_, this| {
            Ok(this.application_timeout.as_millis())
        });
        fields.add_field_method_set("application_timeout", |_, this, val: u64| {
            this.application_timeout = Duration::from_millis(val);
            Ok(())
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function::<_, String, _, _>("key_exists", |_, key| {
            let res = match key.as_str() {
                "update_interval" => true,
                "applications_file" => true,
                "application_timeout" => true,
                _ => false,
            };
            Ok(res)
        });
    }
}