use log::{error, warn};
use mlua::{
    chunk, AnyUserData, AnyUserDataExt, Function, Lua, OwnedTable, Table, TableExt, UserData, Value,
};

use crate::{
    common::{cleanup_guard::CleanupGuard, common::exec_file},
    create_table,
    script_handler::operations::load_operations,
    settings::settings::Settings,
};

pub struct ScriptHandler {
    environment: OwnedTable,
}

impl ScriptHandler {
    pub fn load(lua: &Lua) -> CleanupGuard {
        load_operations(lua);

        let environment = Self::make_sandbox(lua).into_owned();
        exec_file(
            lua,
            &Settings::get().scripts_file,
            environment.clone().to_ref(),
        );

        lua.globals()
            .set("SCRIPT_HANDLER", ScriptHandler { environment })
            .unwrap();

        CleanupGuard::with_name(lua, "SCRIPT_HANDLER")
    }

    fn make_sandbox(lua: &Lua) -> Table {
        let register_fn = lua.create_function(Self::register).unwrap();

        create_table!(lua, {
            ipairs = ipairs,
            next = next,
            pairs = pairs,
            pcall = pcall,
            tonumber = tonumber,
            tostring = tostring,
            type = type,
            unpack = unpack,
            coroutine = { create = coroutine.create, resume = coroutine.resume, running = coroutine.running, status = coroutine.status, wrap = coroutine.wrap },
            string = { byte = string.byte, char = string.char, find = string.find, format = string.format, gmatch = string.gmatch, gsub = string.gsub, len = string.len, lower = string.lower, match = string.match, rep = string.rep, reverse = string.reverse, sub = string.sub, upper = string.upper },
            table = { insert = table.insert, maxn = table.maxn, remove = table.remove, sort = table.sort },
            math = { abs = math.abs, acos = math.acos, asin = math.asin, atan = math.atan, atan2 = math.atan2, ceil = math.ceil, cos = math.cos, cosh = math.cosh, deg = math.deg, exp = math.exp, floor = math.floor, fmod = math.fmod, frexp = math.frexp, huge = math.huge, ldexp = math.ldexp, log = math.log, log10 = math.log10, max = math.max, min = math.min, modf = math.modf, pi = math.pi, pow = math.pow, rad = math.rad, random = math.random, sin = math.sin, sinh = math.sinh, sqrt = math.sqrt, tan = math.tan, tanh = math.tanh },
            os = { clock = os.clock, difftime = os.difftime, time = os.time },
            register = $register_fn,
            print = print,
            Point = OPERATIONS.Point,
            Size = OPERATIONS.Size,
            Rectangle = OPERATIONS.Rectangle,
            Bar = OPERATIONS.Bar,
            Text = OPERATIONS.Text,
            ScrollingText = OPERATIONS.ScrollingText,
            Modifiers = OPERATIONS.Modifiers,
        })
    }

    fn register(
        lua: &Lua,
        (func, sensitivity_list, screens): (Function, Vec<String>, Vec<String>),
    ) -> mlua::Result<()> {
        let screens_object: AnyUserData = lua.globals().get("SCREENS").unwrap();
        let mut found_screens = Vec::new();
        for name in screens {
            let screen: Value = screens_object.call_method("load_screen", name.clone())?;
            match screen {
                Value::Nil => warn!("Could not load screen {}", name),
                Value::LightUserData(screen) => found_screens.push(screen),
                _ => error!("Unexpected error when loading screen"),
            };
        }

        let update_handler_object: Table = lua.globals().get("UPDATE_HANDLER").unwrap();
        update_handler_object.call_method::<_, ()>(
            "register_user_script",
            (func, sensitivity_list, found_screens),
        )?;

        Ok(())
    }
}

impl UserData for ScriptHandler {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("env", |_, this| Ok(this.environment.clone()))
    }
}
