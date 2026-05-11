#[cfg(feature = "from-lua-value")]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use mlua::{FromLua, Lua, UserData, Value, chunk};
    use omni_led_derive::FromLuaValue;

    #[derive(FromLuaValue, Clone, Debug, PartialEq)]
    struct Test {
        a: i32,
        b: String,
        c: bool,
    }
    impl UserData for Test {}

    #[test]
    fn basic_round_trip() {
        let lua = Lua::new();
        let value = Test {
            a: 42,
            b: "It works".to_string(),
            c: true,
        };

        let expected = value.clone();
        let result = lua
            .load(chunk! {
                $value
            })
            .eval::<Test>()
            .unwrap();

        assert_eq!(expected, result);
    }

    #[test]
    #[should_panic = "context: \"Error occurred when parsing 'Test.a'\", cause: FromLuaConversionError"]
    fn field_error() {
        let lua = Lua::new();
        lua.load(chunk! {
            {
                a = "that's not right",
                b = "b",
                c = true
            }
        })
        .eval::<Test>()
        .unwrap();
    }

    fn extract_error_fields(msg: &str) -> Vec<&str> {
        let re = regex::Regex::new(r"\[([^\]]+)\]").unwrap();
        let captured = re
            .captures(&msg)
            .expect("Failed to extract error fields")
            .get(1)
            .expect("Failed to extract error fields")
            .as_str();
        let mut fields: Vec<&str> = captured.split(", ").collect();
        fields.sort_unstable();
        fields
    }

    #[test]
    fn missing_fields() {
        let lua = Lua::new();
        let err = lua
            .load(chunk! {
                {
                    b = "b",
                }
            })
            .eval::<Test>()
            .expect_err("Fail due to fields 'a' and 'c' missing")
            .to_string();

        let expected = "Error occurred when parsing 'Test'\nruntime error: Missing fields: ";
        assert!(
            err.starts_with(expected),
            "Expected: '{expected}'.\nActual:   '{err}'"
        );
        assert_eq!(["a", "c"], *extract_error_fields(&err));
    }

    #[test]
    fn unknown_fields() {
        let lua = Lua::new();
        let err = lua
            .load(chunk! {
                {
                    a = 1,
                    b = "b",
                    c = true,
                    d = "unknown",
                    e = "fields",
                }
            })
            .eval::<Test>()
            .expect_err("Fail due to extra 'd' and 'e' fields")
            .to_string();

        let expected = "Error occurred when parsing 'Test'\nruntime error: Unknown fields: ";
        assert!(
            err.starts_with(expected),
            "Expected: '{expected}'.\nActual:   '{err}'"
        );
        assert_eq!(["d", "e"], *extract_error_fields(&err));
    }

    /// Test proper field cleanup on error. Uninitialized fields must not be dropped.
    /// By trying to parse a struct with N fields and failing on field no. N - 1 we simulate
    /// a situation where one of the fields fails to parse.
    /// To do this, because of the non-deterministic ordering of fields in lua tables, we track
    /// current parse count and drop count to assert them at the end.
    #[test]
    fn error_cleanup() {
        const N: usize = 4;

        static PARSE_COUNT: AtomicUsize = AtomicUsize::new(0);
        static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        #[derive(Debug)]
        struct Tracked;

        impl Clone for Tracked {
            fn clone(&self) -> Self {
                panic!("Should not clone during this test");
            }
        }

        impl Drop for Tracked {
            fn drop(&mut self) {
                DROP_COUNT.fetch_add(1, Ordering::Release);
            }
        }

        impl FromLua for Tracked {
            fn from_lua(_value: Value, _lua: &Lua) -> mlua::Result<Self> {
                let parsed = PARSE_COUNT.fetch_add(1, Ordering::Release);
                if parsed == N - 1 {
                    // This is the last field, time to fail
                    Err(mlua::Error::runtime("Failed due to failure"))
                } else {
                    Ok(Tracked)
                }
            }
        }

        #[derive(FromLuaValue, Clone, Debug)]
        struct CleanupTest {
            a: Tracked,
            b: Tracked,
            c: Tracked,
            d: Tracked,
        }

        let lua = Lua::new();
        lua.load(chunk! {
            {
                a = {},
                b = {},
                c = {},
                d = {},
            }
        })
        .eval::<CleanupTest>()
        .expect_err("Fail due to simulated parse failure");

        assert_eq!(N, CleanupTest::__MASK_MAP.len());
        assert_eq!(N, PARSE_COUNT.load(Ordering::Acquire));
        assert_eq!(N - 1, DROP_COUNT.load(Ordering::Acquire));
    }

    #[derive(FromLuaValue, Clone, Debug, PartialEq)]
    #[mlua(impl_default)]
    struct DefaultTestInner {
        #[mlua(default = 24)]
        a: i32,
        #[mlua(default = 501)]
        b: i32,
    }

    #[derive(FromLuaValue, Clone, Debug, PartialEq)]
    #[mlua(impl_default)]
    struct DefaultTest {
        #[mlua(default = 42)]
        a: i32,
        #[mlua(default = String::from("Default string"))]
        b: String,
        #[mlua(default)]
        c: DefaultTestInner,
    }

    #[test]
    fn rust_default() {
        let expected = DefaultTest {
            a: 42,
            b: String::from("Default string"),
            c: DefaultTestInner { a: 24, b: 501 },
        };
        let result = DefaultTest::default();
        assert_eq!(expected, result);
    }

    #[test]
    fn from_lua_default() {
        let lua = Lua::new();

        let expected = DefaultTest::default();
        let result = lua
            .load(chunk! {
                // Empty table - will default initialize all optional fields
                { }
            })
            .eval::<DefaultTest>()
            .unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn from_lua_override_default() {
        let lua = Lua::new();

        let expected = DefaultTest {
            a: 1,
            b: String::from("2"),
            c: DefaultTestInner { a: 3, b: 4 },
        };
        let result = lua
            .load(chunk! {
                {
                    a = 1,
                    b = '2',
                    c = {
                        a = 3,
                        b = 4,
                    },
                }
            })
            .eval::<DefaultTest>()
            .unwrap();
        assert_eq!(expected, result);
    }
}
