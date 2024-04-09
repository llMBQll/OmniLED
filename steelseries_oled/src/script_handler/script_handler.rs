use mlua::{
    chunk, AnyUserData, AnyUserDataExt, ErrorContext, FromLua, Function, Lua, LuaSerdeExt,
    OwnedFunction, OwnedTable, Table, UserData, UserDataMethods, Value,
};
use oled_derive::FromLuaTable;
use std::time::Duration;

use crate::common::{common::exec_file, scoped_value::ScopedValue};
use crate::create_table;
use crate::model::operation::Operation;
use crate::renderer::renderer::{ContextKey, Renderer};
use crate::screen::screens::LuaScreenWrapper;
use crate::script_handler::operations::load_operations;
use crate::settings::settings::{get_full_path, Settings};

pub struct ScriptHandler {
    environment: OwnedTable,
    renderer: Renderer,
    screens: Vec<ScreenContext>,
}

struct ScreenContext {
    screen: LuaScreenWrapper,
    scripts: Vec<UserScript>,
    marked_for_update: Vec<bool>,
    time_remaining: Duration,
    last_priority: usize,
    repeats: Option<Repeat>,
    index: usize,
}

const DEFAULT_UPDATE_TIME: Duration = Duration::from_millis(1000);

impl ScriptHandler {
    pub fn load(lua: &Lua) -> ScopedValue {
        load_operations(lua);

        let environment = Self::make_sandbox(lua);

        let value = ScopedValue::new(
            lua,
            "SCRIPT_HANDLER",
            ScriptHandler {
                renderer: Renderer::new(),
                environment: environment.clone().into_owned(),
                screens: vec![],
            },
        );

        exec_file(
            lua,
            &get_full_path(&Settings::get().scripts_file),
            environment,
        );

        value
    }

    fn register(
        &mut self,
        lua: &Lua,
        screen_name: String,
        user_scripts: Vec<UserScript>,
    ) -> mlua::Result<()> {
        let screens_object: AnyUserData = lua.globals().get("SCREENS").unwrap();
        let screen: LuaScreenWrapper = screens_object.call_method("load", screen_name)?;

        let screen_count = self.screens.len();
        let script_count = user_scripts.len();

        let context = ScreenContext {
            screen,
            scripts: user_scripts,
            marked_for_update: vec![false; script_count],
            time_remaining: Default::default(),
            last_priority: 0,
            repeats: None,
            index: screen_count,
        };
        self.screens.push(context);

        Ok(())
    }

    fn set_value(
        &mut self,
        lua: &Lua,
        application_name: String,
        event: String,
        data: Value,
    ) -> mlua::Result<()> {
        let env = self.environment.to_ref();

        if !env.contains_key(application_name.clone()).unwrap() {
            env.set(application_name.clone(), lua.create_table().unwrap())
                .unwrap();
        }

        let entry: Table = env.get(application_name.clone()).unwrap();
        entry.set(event.clone(), data.clone()).unwrap();

        let key = format!("{}.{}", application_name, event);
        for screen in &mut self.screens {
            for (index, script) in screen.scripts.iter().enumerate() {
                if script.run_on.contains(&key) {
                    screen.marked_for_update[index] = true;
                }
            }
        }

        Ok(())
    }

    fn update(&mut self, lua: &Lua, time_passed: Duration) -> mlua::Result<()> {
        let env = self.environment.to_ref();
        for screen in &mut self.screens {
            Self::update_impl(lua, screen, &mut self.renderer, &env, time_passed)?;
        }
        Ok(())
    }

    fn update_impl(
        lua: &Lua,
        ctx: &mut ScreenContext,
        renderer: &mut Renderer,
        env: &Table,
        time_passed: Duration,
    ) -> mlua::Result<()> {
        ctx.time_remaining = ctx.time_remaining.saturating_sub(time_passed);

        let mut marked_for_update = vec![false; ctx.marked_for_update.len()];
        std::mem::swap(&mut ctx.marked_for_update, &mut marked_for_update);

        let mut to_update = None;
        let mut update_modifier = None;
        for (priority, marked_for_update) in marked_for_update.into_iter().enumerate() {
            if !ctx.time_remaining.is_zero() && ctx.last_priority < priority {
                if let Some(Repeat::ToFit) = ctx.repeats {
                    to_update = Some(ctx.last_priority);
                    update_modifier = Some(Repeat::ToFit);
                }
                break;
            }

            if ctx.last_priority == priority && ctx.repeats == Some(Repeat::Once) {
                to_update = Some(ctx.last_priority);
                update_modifier = Some(Repeat::Once);
                break;
            }

            let predicate = match &ctx.scripts[priority].predicate {
                Some(predicate) => predicate.call::<_, bool>(()).unwrap(),
                None => true,
            };
            if marked_for_update && predicate {
                to_update = Some(priority);
                update_modifier = None;
                break;
            }
        }

        let to_update = match to_update {
            Some(to_update) => to_update,
            None => return Ok(()),
        };

        let size = ctx.screen.get().borrow_mut().size(lua).unwrap(); // TODO handle errors
        env.set("SCREEN", size)?;

        let output: ScriptOutput = ctx.scripts[to_update].action.call(())?;
        let (end_auto_repeat, image) = renderer.render(
            ContextKey {
                script: to_update,
                screen: ctx.index,
            },
            size,
            output.data,
        );

        ctx.screen.get().borrow_mut().update(lua, image).unwrap(); // TODO handle errors

        ctx.repeats = match (output.repeats, end_auto_repeat) {
            (Some(Repeat::ToFit), _) => Some(Repeat::ToFit),
            (Some(Repeat::Once), false) => Some(Repeat::Once),
            (_, _) => None,
        };
        ctx.time_remaining = match update_modifier {
            Some(Repeat::ToFit) => ctx.time_remaining,
            _ => output.duration,
        };
        ctx.last_priority = to_update;

        Ok(())
    }

    fn reset(&mut self) -> mlua::Result<()> {
        for screen in &mut self.screens {
            screen.marked_for_update = vec![false; screen.marked_for_update.len()];
            screen.time_remaining = Duration::ZERO;
            screen.last_priority = 0;
            screen.repeats = None;
        }

        Ok(())
    }

    fn make_sandbox(lua: &Lua) -> Table {
        create_table!(lua, {
            register = function(screen, user_scripts)
                SCRIPT_HANDLER:register(screen, user_scripts)
            end,
            Point = OPERATIONS.Point,
            Size = OPERATIONS.Size,
            Rectangle = OPERATIONS.Rectangle,
            Bar = OPERATIONS.Bar,
            Text = OPERATIONS.Text,
            ScrollingText = OPERATIONS.ScrollingText,
            Modifiers = OPERATIONS.Modifiers,
            EVENTS = EVENTS,
            LOG = LOG,
            PLATFORM = PLATFORM,
            SHORTCUTS = SHORTCUTS,
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
}

impl UserData for ScriptHandler {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut(
            "register",
            |lua, handler, (screen, user_scripts): (String, Vec<UserScript>)| {
                handler.register(lua, screen, user_scripts)
            },
        );

        methods.add_method_mut("update", |lua, handler, time_passed: u64| {
            handler.update(lua, Duration::from_millis(time_passed))
        });

        methods.add_method_mut(
            "set_value",
            |lua, handler, (application_name, event, data): (String, String, Value)| {
                handler.set_value(lua, application_name, event, data)
            },
        );

        methods.add_method_mut("reset", |_lua, handler, _: ()| handler.reset());
    }
}

#[derive(FromLuaTable, Clone)]
struct UserScript {
    #[mlua(transform(Self::transform_function))]
    action: OwnedFunction,

    #[mlua(transform(Self::transform_function_option))]
    predicate: Option<OwnedFunction>,

    run_on: Vec<String>,
}

impl UserScript {
    fn transform_function(function: Function, _lua: &Lua) -> mlua::Result<OwnedFunction> {
        Ok(function.into_owned())
    }

    fn transform_function_option(
        function: Option<Function>,
        _lua: &Lua,
    ) -> mlua::Result<Option<OwnedFunction>> {
        Ok(function.and_then(|p| Some(p.into_owned())))
    }
}

#[derive(Debug, PartialEq, Copy, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Repeat {
    Once,
    ToFit,
}

#[derive(FromLuaTable, Clone)]
struct ScriptOutput {
    data: Vec<Operation>,

    #[mlua(transform(Self::transform_duration))]
    duration: Duration,

    #[mlua(transform(Self::transform_repeats))]
    repeats: Option<Repeat>,
}

impl ScriptOutput {
    fn transform_duration(duration: Option<u64>, _lua: &Lua) -> mlua::Result<Duration> {
        let duration = duration
            .and_then(|duration| Some(Duration::from_millis(duration)))
            .unwrap_or(DEFAULT_UPDATE_TIME);
        Ok(duration)
    }

    fn transform_repeats(repeats: Value, lua: &Lua) -> mlua::Result<Option<Repeat>> {
        lua.from_value(repeats)
    }
}
