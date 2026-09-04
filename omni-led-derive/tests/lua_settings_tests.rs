#[cfg(all(feature = "lua-settings", feature = "lua-name"))]
mod tests {
    use mlua::{IntoLua, Lua, Value, chunk};
    use omni_led_derive::LuaSettings;
    use std::cell::RefCell;

    thread_local! {
        static EVENTS: RefCell<Vec<(String, Value)>> = RefCell::new(Vec::new());
    }

    fn clear_events() {
        EVENTS.with(|e| e.borrow_mut().clear());
    }

    fn take_events() -> Vec<(String, Value)> {
        EVENTS.with(|e| std::mem::take(&mut *e.borrow_mut()))
    }

    fn send_test_event<T: IntoLua + Clone>(
        lua: &Lua,
        event_name: &str,
        value: &T,
    ) -> mlua::Result<()> {
        let lua_value = value.clone().into_lua(lua)?;
        EVENTS.with(|events| {
            events
                .borrow_mut()
                .push((event_name.to_string(), lua_value));
        });
        Ok(())
    }

    #[derive(LuaSettings, Clone)]
    #[omni(root = TestSettings)]
    #[omni(on_set = send_test_event)]
    struct TestSettings {
        a: u32,
    }

    #[derive(LuaSettings, Clone)]
    #[omni(root = TestSettings)]
    #[omni(on_set = send_test_event)]
    struct TestSettingsOuter {
        a: u32,
        inner: TestSettingsInner,
    }

    #[derive(LuaSettings, Clone)]
    #[omni(root = TestSettings.inner)]
    #[omni(on_set = send_test_event)]
    struct TestSettingsInner {
        a: u32,
    }

    fn env<S: IntoLua>(settings_str: &'static str, settings: S) -> Lua {
        clear_events();

        let lua = Lua::new();
        let value = settings.into_lua(&lua).unwrap();
        lua.globals().set(settings_str, value).unwrap();
        lua
    }

    #[test]
    fn send_event() {
        const SETTINGS_STR: &str = "TestSettings";
        let lua = env(SETTINGS_STR, TestSettings { a: 7 });

        lua.load(chunk! {
            TestSettings.a = 7
            TestSettings.a = 7
        })
        .exec()
        .unwrap();

        // Events sent despite not actually changing the value,
        // and even when set twice in a row
        assert_eq!(
            take_events(),
            [
                ("TestSettings.a".to_owned(), Value::Integer(7)),
                ("TestSettings.a".to_owned(), Value::Integer(7))
            ]
        );
    }

    #[test]
    fn send_event_nested() {
        const SETTINGS_STR: &str = "TestSettings";
        let lua = env(
            SETTINGS_STR,
            TestSettingsOuter {
                a: 7,
                inner: TestSettingsInner { a: 7 },
            },
        );

        lua.load(chunk! {
            TestSettings.a = 7
            TestSettings.inner.a = 7
        })
        .exec()
        .unwrap();

        assert_eq!(
            take_events(),
            [
                ("TestSettings.a".to_owned(), Value::Integer(7)),
                ("TestSettings.inner.a".to_owned(), Value::Integer(7))
            ]
        );
    }
}
