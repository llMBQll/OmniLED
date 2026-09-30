use mlua::{FromLua, FromLuaMulti, Function, IntoLuaMulti, Lua, MetaMethod, Table, Value};
use std::os::raw::c_void;

use crate::common::lua_access_handler::LuaAccessHandler;
use crate::common::lua_register::{set_lua_enums, set_lua_types};

#[macro_export]
macro_rules! sandbox_value {
    ($dst_src:expr) => {
        crate::common::sandbox::SandboxValue::Global {
            dst: $dst_src,
            src: $dst_src,
        }
    };
    ($dst:expr, src: $src:expr) => {
        crate::common::sandbox::SandboxValue::Global {
            dst: $dst,
            src: $src,
        }
    };
    ($dst:expr, val: $val:expr) => {
        crate::common::sandbox::SandboxValue::Value {
            dst: $dst,
            val: $val,
        }
    };
    ($dst:expr, function: $val:expr) => {
        crate::common::sandbox::SandboxValue::Value {
            dst: $dst,
            val: mlua::Value::Function($val),
        }
    };
    ($dst:expr, table: $val:expr) => {
        crate::common::sandbox::SandboxValue::Value {
            dst: $dst,
            val: mlua::Value::Table($val),
        }
    };
}

#[derive(Clone)]
pub struct Sandbox {
    pub real: Table,
    pub proxy: Table,
}

impl Sandbox {
    pub fn new(lua: &Lua, values: Vec<SandboxValue>) -> Self {
        let real = lua.create_table().unwrap();

        let proxy_meta = lua.create_table().unwrap();

        // With this metatable `_G` will mostly behave as if it was the real one from user script prerspective
        let real_env = real.clone();
        let mut handler = LuaAccessHandler::instance(lua);
        proxy_meta
            .set(
                MetaMethod::Index.name(),
                lua.create_function_mut(move |lua, (_, key): (Table, String)| {
                    handler.get(lua, real_env.clone(), &key)
                })
                .unwrap(),
            )
            .unwrap();

        let real_env = real.clone();
        let mut handler = LuaAccessHandler::instance(lua);
        proxy_meta
            .set(
                MetaMethod::NewIndex.name(),
                lua.create_function_mut(move |lua, (_, key, value): (Table, String, Value)| {
                    handler.set(lua, real_env.clone(), &key, value)
                })
                .unwrap(),
            )
            .unwrap();

        let real_env = real.clone();
        proxy_meta
            .set(
                MetaMethod::Len.name(),
                lua.create_function(move |_, _: Table| real_env.len())
                    .unwrap(),
            )
            .unwrap();

        let real_env = real.clone();
        let next = lua.globals().get::<Function>("next").unwrap();
        proxy_meta
            .set(
                MetaMethod::Pairs.name(),
                lua.create_function(move |_, _: Table| {
                    // Reference: https://www.lua.org/manual/5.4/manual.html#pdf-pairs
                    Ok((next.clone(), real_env.clone(), Value::Nil))
                })
                .unwrap(),
            )
            .unwrap();

        // Make sure metatable cannot be changed from user scripts
        proxy_meta.set("__metatable", "protected").unwrap();

        let proxy = lua.create_table().unwrap();
        proxy.set_metatable(Some(proxy_meta)).unwrap();

        fill_sandbox_env(lua, values, &real, &proxy).unwrap();

        Self { real, proxy }
    }
}

pub enum SandboxValue {
    Global {
        dst: &'static str,
        src: &'static str,
    },
    Value {
        dst: &'static str,
        val: Value,
    },
}

fn fill_sandbox_env(
    lua: &Lua,
    user_values: Vec<SandboxValue>,
    real: &Table,
    proxy: &Table,
) -> mlua::Result<()> {
    let default_values = vec![
        sandbox_value!("dump", function: wrap_fn(lua, dump)),
        sandbox_value!("_G", table: proxy.clone()),
        sandbox_value!("getmetatable"),
        sandbox_value!("ipairs"),
        sandbox_value!("marked_table", function: wrap_fn(lua, marked_table)),
        sandbox_value!("next"),
        sandbox_value!("pairs"),
        sandbox_value!("pcall"),
        sandbox_value!("print"),
        sandbox_value!("rawequal"),
        sandbox_value!("rawget", function: wrap_raw_fn(lua, &proxy, safe_raw_get)),
        sandbox_value!("rawlen", function: wrap_raw_fn(lua, &proxy, safe_raw_len)),
        sandbox_value!("rawset", function: wrap_raw_fn(lua, &proxy, safe_raw_set)),
        sandbox_value!("select"),
        sandbox_value!("setmetatable"),
        sandbox_value!("tonumber"),
        sandbox_value!("tostring"),
        sandbox_value!("type"),
        sandbox_value!("_VERSION"),
        sandbox_value!("xpcall"),
        sandbox_value!("coroutine"),
        sandbox_value!("math"),
        sandbox_value!("math.round", function: wrap_fn(lua, round)),
        sandbox_value!("os.clock"),
        sandbox_value!("os.date"),
        sandbox_value!("os.difftime"),
        sandbox_value!("os.getenv"),
        sandbox_value!("os.time"),
        sandbox_value!("string"),
        sandbox_value!("utf8"),
        sandbox_value!("table"),
    ];

    let globals = lua.globals();
    for value in default_values.into_iter().chain(user_values.into_iter()) {
        match value {
            SandboxValue::Global { dst, src } => {
                let val = get_value::<Value>(globals.clone(), src)?
                    .ok_or_else(|| mlua::Error::runtime(format!("'{}' not found", src)))?;
                set_value(lua, real.clone(), dst, val, true)?;
            }
            SandboxValue::Value { dst, val } => {
                set_value(lua, real.clone(), dst, val, false)?;
            }
        }
    }

    set_lua_enums(lua, real);
    set_lua_types(lua, real);

    Ok(())
}

fn get_value<V: FromLua>(mut env: Table, key: &str) -> mlua::Result<Option<V>> {
    let mut parts = key.split('.').peekable();
    while let Some(key) = parts.next() {
        if parts.peek().is_none() {
            return env.get(key);
        }
        env = env.get(key)?;
    }
    Ok(None)
}

fn set_value(lua: &Lua, mut parent: Table, key: &str, val: Value, deep: bool) -> mlua::Result<()> {
    let mut parts = key.split('.').peekable();

    while let Some(key) = parts.next() {
        if parts.peek().is_none() {
            if deep {
                return deep_copy(lua, &parent, key, val);
            } else {
                return parent.set(key, val);
            }
        }

        let new_parent: Option<Table> = parent.get(key)?;
        let new_parent = match new_parent {
            Some(new_parent) => new_parent,
            None => {
                let new_parent = lua.create_table()?;
                parent.set(key, new_parent.clone())?;
                new_parent
            }
        };
        parent = new_parent;
    }

    Ok(())
}

fn deep_copy(lua: &Lua, parent: &Table, key: &str, val: Value) -> mlua::Result<()> {
    match val {
        Value::Table(table) => {
            let new_parent = lua.create_table()?;
            new_parent.set_metatable(table.metatable()).unwrap();
            for pair in table.pairs::<String, Value>() {
                let (key, val) = pair?;
                deep_copy(lua, &new_parent, &key, val)?;
            }
            parent.set(key, new_parent)
        }
        val => parent.set(key, val),
    }
}

// Wrap raw functions in a way that do not bypass the proxy table metatable
fn wrap_raw_fn<A: FromLuaMulti + 'static, R: IntoLuaMulti + 'static>(
    lua: &Lua,
    proxy: &Table,
    raw: fn(*const c_void, A) -> mlua::Result<R>,
) -> Function {
    let proxy_ptr = proxy.to_pointer();
    lua.create_function(move |_lua, args: A| raw(proxy_ptr, args))
        .unwrap()
}

fn safe_raw_get(proxy_ptr: *const c_void, (target, key): (Table, Value)) -> mlua::Result<Value> {
    if target.to_pointer() == proxy_ptr {
        println!("_");
        target.get(key)
    } else {
        println!("__");
        target.raw_get(key)
    }
}

fn safe_raw_len(proxy_ptr: *const c_void, target: Table) -> mlua::Result<i64> {
    if target.to_pointer() == proxy_ptr {
        target.len()
    } else {
        Ok(target.raw_len() as i64)
    }
}

fn safe_raw_set(
    proxy_ptr: *const c_void,
    (target, key, val): (Table, Value, Value),
) -> mlua::Result<()> {
    if target.to_pointer() == proxy_ptr {
        target.set(key, val)
    } else {
        target.raw_set(key, val)
    }
}

// Extra functions
fn wrap_fn<A: FromLuaMulti + 'static, R: IntoLuaMulti + 'static>(
    lua: &Lua,
    func: fn(&Lua, A) -> mlua::Result<R>,
) -> Function {
    lua.create_function(move |lua, args: A| func(lua, args))
        .unwrap()
}

fn dump(_: &Lua, val: Value) -> mlua::Result<String> {
    let string = format!("{:#?}", val);
    Ok(string)
}

fn marked_table(lua: &Lua, table: Table) -> mlua::Result<Table> {
    crate::events::cbor_to_lua::marked_table(lua, table)
}

fn round(_: &Lua, val: f64) -> mlua::Result<i64> {
    let value = val.round() as i64;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use mlua::{IntoLua, chunk};
    use std::{cell::RefCell, rc::Rc};

    use super::*;

    macro_rules! eval {
        ($lua:expr, $proxy:expr, $code:tt) => {
            $lua.load(chunk! $code)
                .set_environment($proxy.clone())
                .eval()
                .unwrap()
        };
    }

    macro_rules! exec {
        ($lua:expr, $proxy:expr, $code:tt) => {
            $lua.load(chunk! $code)
                .set_environment($proxy.clone())
                .exec()
                .unwrap()
        };
    }

    #[test]
    fn get_global_metatable() {
        let lua = Lua::new();
        let sandbox = Sandbox::new(&lua, vec![]);

        let metatable: String = eval!(lua, sandbox.proxy, { getmetatable(_G) });
        assert_eq!(metatable, "protected");
    }

    #[test]
    #[should_panic = "cannot change a protected metatable"]
    fn set_global_metatable() {
        let lua = Lua::new();
        let sandbox = Sandbox::new(&lua, vec![]);

        exec!(lua, sandbox.proxy, { setmetatable(_G, {}) });
    }

    #[test]
    fn global_pairs() {
        let lua = Lua::new();
        let sandbox = Sandbox::new(&lua, vec![]);

        let (result, len): (Table, i64) = eval!(lua, sandbox.proxy, {
            local result = {}
            local len = 0
            for k, v in pairs(_G) do
                result[k] = v
                len = len + 1
            end
            return result, len
        });

        assert_ne!(len, 0);
        for item in result.pairs::<Value, Value>() {
            let (k, v) = item.unwrap();
            assert_eq!(sandbox.real.get::<Value>(k).unwrap(), v);
        }
    }

    #[test]
    fn safe_raw_get() {
        const VERSION: &str = "Lua 5.4";

        let lua = Lua::new();

        let non_g = lua.create_table().unwrap();
        let sandbox = Sandbox::new(&lua, vec![sandbox_value!("_NON_G", table: non_g.clone())]);

        // Reuse the proxy metatable to prove only the _G table is special
        non_g
            .set_metatable(sandbox.proxy.metatable().clone())
            .unwrap();

        // Lua side cannot bypass the metatable for _G
        let version: String = eval!(lua, sandbox.proxy, { return rawget(_G, "_VERSION") });
        assert_eq!(version, VERSION);

        // Lua side cannot bypass the metatable for copy of _G
        let version: String = eval!(lua, sandbox.proxy, {
             local g = _G
             return rawget(g, "_VERSION")
        });
        assert_eq!(version, VERSION);

        // Now try regular get non-special table
        let version: String = eval!(lua, sandbox.proxy, { return _NON_G["_VERSION"] });
        assert_eq!(version, VERSION);

        // Now try raw get non-special table - should skip metatable
        let version: Value = eval!(lua, sandbox.proxy, { return rawget(_NON_G, "_VERSION") });
        assert_eq!(version, Value::Nil);
    }

    #[test]
    fn safe_raw_set() {
        const REAL_VERSION: &str = "Lua 5.4";
        const TEST_VERSION: &str = "Test Lua 5.4";

        let lua = Lua::new();

        let non_g = lua.create_table().unwrap();
        let test_version = TEST_VERSION.into_lua(&lua).unwrap();
        let sandbox = Sandbox::new(
            &lua,
            vec![
                sandbox_value!("_NON_G", table: non_g.clone()),
                sandbox_value!("_TEST_VERSION", val: test_version),
            ],
        );

        // Reuse the proxy metatable to prove only the _G table is special
        non_g
            .set_metatable(sandbox.proxy.metatable().clone())
            .unwrap();

        // Lua side cannot bypass the metatable for _G
        sandbox.real.set("_VERSION", REAL_VERSION).unwrap();
        exec!(lua, sandbox.proxy, {
            rawset(_G, "_VERSION", _TEST_VERSION)
        });
        assert_eq!(
            sandbox.real.get::<String>("_VERSION").unwrap(),
            TEST_VERSION
        );
        assert_eq!(
            sandbox.proxy.raw_get::<Value>("_VERSION").unwrap(),
            Value::Nil
        );

        // Lua side cannot bypass the metatable for copy of _G
        sandbox.real.set("_VERSION", REAL_VERSION).unwrap();
        exec!(lua, sandbox.proxy, {
             local g = _G
             rawset(g, "_VERSION", _TEST_VERSION)
        });
        assert_eq!(
            sandbox.real.get::<String>("_VERSION").unwrap(),
            TEST_VERSION
        );
        assert_eq!(
            sandbox.proxy.raw_get::<Value>("_VERSION").unwrap(),
            Value::Nil
        );

        // Now try regular set non-special table
        sandbox.real.set("_VERSION", REAL_VERSION).unwrap();
        exec!(lua, sandbox.proxy, { _NON_G["_VERSION"] = _TEST_VERSION });
        assert_eq!(
            sandbox.real.get::<String>("_VERSION").unwrap(),
            TEST_VERSION
        );
        assert_eq!(non_g.raw_get::<Value>("_VERSION").unwrap(), Value::Nil);

        // Now try raw set non-special table - should skip metatable
        sandbox.real.set("_VERSION", REAL_VERSION).unwrap();
        exec!(lua, sandbox.proxy, {
            return rawset(_NON_G, "_VERSION", _TEST_VERSION);
        });
        assert_eq!(
            sandbox.real.get::<String>("_VERSION").unwrap(),
            REAL_VERSION
        );
        assert_eq!(non_g.raw_get::<String>("_VERSION").unwrap(), TEST_VERSION);
    }

    #[test]
    fn lua_access_handler() {
        let lua = Lua::new();
        let sandbox = Sandbox::new(&lua, vec![]);

        let gets = Rc::new(RefCell::new(0));
        let sets = Rc::new(RefCell::new(0));

        let mut handler = LuaAccessHandler::instance(&lua);

        handler.add_on_get("tracked", {
            let gets = gets.clone();
            move |_lua, table, key| {
                *gets.borrow_mut() += 1;
                table.get(key)
            }
        });

        handler.add_on_set("tracked", {
            let sets = sets.clone();
            move |_lua, table, key, value| {
                *sets.borrow_mut() += 1;
                table.set(key, value)
            }
        });

        exec!(lua, sandbox.proxy, {
            tracked = 7
            tracked = nil
            tracked = 'A'

            other = tracked
            other = tracked
        });

        assert_eq!(gets.take(), 2);
        assert_eq!(sets.take(), 3);
    }

    fn setup_readonly_env() -> (Lua, Sandbox) {
        let lua = Lua::new();
        let sandbox = Sandbox::new(
            &lua,
            vec![sandbox_value!("my_readonly", val: Value::Integer(7))],
        );

        let mut handler = LuaAccessHandler::instance(&lua);
        handler.add_on_set("my_readonly", move |_lua, _table, key, _value| {
            Err(mlua::Error::runtime(format!("'{key}' is readonly")))
        });

        (lua, sandbox)
    }

    #[test]
    fn lua_readonly_read() {
        let (lua, sandbox) = setup_readonly_env();

        let value: i64 = eval!(lua, sandbox.proxy, { my_readonly });
        assert_eq!(value, 7);
    }

    #[test]
    #[should_panic = "'my_readonly' is readonly"]
    fn lua_readonly_write() {
        let (lua, sandbox) = setup_readonly_env();

        exec!(lua, sandbox.proxy, { my_readonly = 7 });
    }
}
