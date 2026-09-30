#[cfg(feature = "lua-settings")]
mod tests {
    use mlua::{Lua, Table, Value, chunk};
    use omni_led_derive::{FromLuaValue, LuaSettings};

    const RESULTS_KEY: &str = "__results";

    fn test_env() -> Lua {
        let lua = Lua::new();
        let results = lua.create_table().unwrap();
        lua.globals().set(RESULTS_KEY, results).unwrap();
        return lua;
    }

    fn get_results(lua: &Lua) -> Vec<(String, Value)> {
        let results: Table = lua.globals().get(RESULTS_KEY).unwrap();

        let mut results = results
            .pairs::<String, Value>()
            .into_iter()
            .map(|result| {
                let (key, val) = result.unwrap();
                (key, val)
            })
            .collect::<Vec<_>>();
        results.sort_by(|(lhs_key, _), (rhs_key, _)| lhs_key.cmp(rhs_key));
        results
    }

    macro_rules! on_set_fn {
        ($name:literal, $ty:ty) => {
            |lua: &Lua, value: &$ty| -> mlua::Result<()> {
                let results: Table = lua.globals().get(RESULTS_KEY).unwrap();
                results.set($name, value.clone()).unwrap();
                Ok(())
            }
        };
    }

    #[derive(Clone, Debug, LuaSettings, FromLuaValue, PartialEq)]
    struct Test {
        #[omni(on_set = on_set_fn!("a", i32))]
        a: i32,
        #[omni(on_set = on_set_fn!("b", bool))]
        b: bool,
        c: f64,
    }

    #[test]
    fn basic_assign() {
        let lua = test_env();

        let value = Test {
            a: 41,
            b: false,
            c: 6.8,
        };

        let expected_value = Test {
            a: 42,
            b: true,
            c: 6.9,
        };
        let updated_value = lua
            .load(chunk! {
                local value = $value
                value.a = 42
                value.b = true
                value.c = 6.9
                return value
            })
            .eval::<Test>()
            .unwrap();

        assert_eq!(updated_value, expected_value);
        assert_eq!(
            vec![
                ("a".to_string(), Value::Integer(42)),
                ("b".to_string(), Value::Boolean(true)),
                // c is set but it has no `on_set` handler
            ],
            get_results(&lua)
        );
    }

    #[derive(Clone, Debug, LuaSettings, FromLuaValue, PartialEq)]
    struct TestValidate {
        #[omni(on_set = on_set_fn!("a", i32))]
        #[omni(validate = Self::validate_a)]
        a: i32,
    }

    impl TestValidate {
        fn validate_a(_lua: &Lua, a: &i32) -> mlua::Result<()> {
            if *a < 0 || *a > 10 {
                Err(mlua::Error::runtime(format!(
                    "Expected value in range [0, 10], got '{}'",
                    *a
                )))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn validate() {
        let lua = test_env();

        let value = TestValidate { a: 5 };
        let expected_value = TestValidate { a: 10 };
        let result = lua
            .load(chunk! {
                local value = $value
                value.a = 10
                return value
            })
            .eval::<TestValidate>()
            .unwrap();

        assert_eq!(result, expected_value);
        assert_eq!(
            vec![("a".to_string(), Value::Integer(10)),],
            get_results(&lua)
        );

        let value = TestValidate { a: 5 };
        lua.load(chunk! {
            local value = $value
            value.a = 11
            return value
        })
        .eval::<TestValidate>()
        .expect_err("Validate expected to fail");
    }

    #[derive(Clone, Debug, LuaSettings, FromLuaValue, PartialEq)]
    struct TestRecursiveA {
        #[omni(on_set = on_set_fn!("A.a", i32))]
        a: i32,

        #[omni(on_set = TestRecursiveB::recursive_set)]
        b: TestRecursiveB,
    }

    #[derive(Clone, Debug, LuaSettings, FromLuaValue, PartialEq)]
    struct TestRecursiveB {
        #[omni(on_set = on_set_fn!("B.a", i32))]
        a: i32,

        #[omni(on_set = TestRecursiveC::recursive_set)]
        c: TestRecursiveC,
    }

    #[derive(Clone, Debug, LuaSettings, FromLuaValue, PartialEq)]
    struct TestRecursiveC {
        #[omni(on_set = on_set_fn!("C.a", i32))]
        a: i32,
    }

    #[test]
    fn recursive_set() {
        let lua = test_env();

        // In the real code this will be called after a lua value got
        // assigned or constructed using `a = new_value`
        // TODO test this directly by recreating special lua env
        TestRecursiveA::recursive_set(
            &lua,
            &TestRecursiveA {
                a: 1,
                b: TestRecursiveB {
                    a: 2,
                    c: TestRecursiveC { a: 3 },
                },
            },
        )
        .unwrap();

        assert_eq!(
            vec![
                ("A.a".to_string(), Value::Integer(1)),
                ("B.a".to_string(), Value::Integer(2)),
                ("C.a".to_string(), Value::Integer(3)),
            ],
            get_results(&lua)
        );
    }
}
