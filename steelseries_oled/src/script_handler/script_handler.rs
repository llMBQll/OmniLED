use log::{error, warn};
use mlua::{
    chunk, AnyUserData, AnyUserDataExt, Function, Lua, OwnedTable, Table, TableExt, UserData, Value,
};

use crate::settings::settings::get_full_path;
use crate::{
    common::{common::exec_file, scoped_value::ScopedValue},
    create_table,
    script_handler::operations::load_operations,
    settings::settings::Settings,
};

pub struct ScriptHandler {
    environment: OwnedTable,
}

impl ScriptHandler {
    pub fn load(lua: &Lua) -> ScopedValue {
        load_operations(lua);

        let environment = Self::make_sandbox(lua).into_owned();
        exec_file(
            lua,
            &get_full_path(&Settings::get().scripts_file),
            environment.clone().to_ref(),
        );

        ScopedValue::new(lua, "SCRIPT_HANDLER", ScriptHandler { environment })
    }

    fn make_sandbox(lua: &Lua) -> Table {
        let register_fn = lua.create_function(Self::register).unwrap();

        create_table!(lua, {
            register = $register_fn,
            Point = OPERATIONS.Point,
            Size = OPERATIONS.Size,
            Rectangle = OPERATIONS.Rectangle,
            Bar = OPERATIONS.Bar,
            Text = OPERATIONS.Text,
            ScrollingText = OPERATIONS.ScrollingText,
            Modifiers = OPERATIONS.Modifiers,
            EVENTS = EVENTS,
            KEY_COMBINATION_HANDLER = KEY_COMBINATION_HANDLER,
            PLATFORM = PLATFORM,
            ipairs = ipairs,
            next = next,
            pairs = pairs,
            pcall = pcall,
            print = print,
            tonumber = tonumber,
            tostring = tostring,
            type = type,
            coroutine = { close = coroutine.close, create = coroutine.create, isyieldable = coroutine.isyieldable, resume = coroutine.resume, running = coroutine.running, status = coroutine.status, wrap = coroutine.wrap, yield = coroutine.yield },
            math = { abs = math.abs, acos = math.acos, asin = math.asin, atan = math.atan, atan2 = math.atan2, ceil = math.ceil, cos = math.cos, cosh = math.cosh, deg = math.deg, exp = math.exp, floor = math.floor, fmod = math.fmod, frexp = math.frexp, huge = math.huge, ldexp = math.ldexp, log = math.log, log10 = math.log10, max = math.max, maxinteger = math.maxinteger, min = math.min, mininteger = math.mininteger, modf = math.modf, pi = math.pi, pow = math.pow, rad = math.rad, random = math.random, randomseed = math.randomseed, sin = math.sin, sinh = math.sinh, sqrt = math.sqrt, tan = math.tan, tanh = math.tanh, tointeger = math.tointeger, type = math.type, ult = math.ult },
            os = { clock = os.clock, date = os.date, difftime = os.difftime, getenv = os.getenv, time = os.time },
            string = { byte = string.byte, char = string.char, dump = string.dump, find = string.find, format = string.format, gmatch = string.gmatch, gsub = string.gsub, len = string.len, lower = string.lower, match = string.match, pack = string.pack, packsize = string.packsize, rep = string.rep, reverse = string.reverse, sub = string.sub, unpack = string.unpack, upper = string.upper },
            table = { concat = table.concat, insert = table.insert, move = table.move, pack = table.pack, remove = table.remove, sort = table.sort, unpack = table.unpack },
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

        let event_handler: Table = lua.globals().get("EVENT_HANDLER").unwrap();
        event_handler.call_method::<_, ()>(
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
