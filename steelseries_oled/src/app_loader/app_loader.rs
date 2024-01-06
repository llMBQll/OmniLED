use log::{error, warn};
use mlua::{chunk, Lua, LuaSerdeExt, MetaMethod, UserData, Value};

use crate::common::scoped_value::ScopedValue;
use crate::settings::settings::get_full_path;
use crate::{
    app_loader::process::Process, common::common::exec_file, create_table,
    settings::settings::Settings,
};

pub struct AppLoader {
    processes: Vec<Process>,
}

impl AppLoader {
    pub fn load(lua: &Lua) -> ScopedValue {
        let app_loader = ScopedValue::new(
            lua,
            "APP_LOADER",
            Self {
                processes: Vec::new(),
            },
        );

        let env = create_table!(lua, {
            load_app = function(config) APP_LOADER:start_process(config) end,
            SERVER = SERVER,
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
        });
        exec_file(lua, &get_full_path(&Settings::get().applications_file), env);

        let len: usize = lua.load(chunk! { #APP_LOADER }).eval().unwrap();
        if len == 0 {
            warn!("App loader didn't load any applications");
        }

        app_loader
    }
}

impl UserData for AppLoader {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("start_process", |lua, this, app_config: Value| {
            let app_config = match lua.from_value(app_config) {
                Ok(app_config) => app_config,
                Err(err) => {
                    error!("Failed to parse process config: {}", err);
                    return Ok(());
                }
            };

            match Process::new(&app_config) {
                Ok(process) => {
                    this.processes.push(process);
                }
                Err(err) => {
                    error!(
                        "Failed to run {}: '{}'",
                        serde_json::to_string(&app_config).unwrap(),
                        err
                    );
                }
            }
            Ok(())
        });

        methods.add_meta_method(MetaMethod::Len, |_, this, ()| Ok(this.processes.len()))
    }
}
