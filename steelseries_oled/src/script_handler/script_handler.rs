use mlua::{
    chunk, AnyUserData, AnyUserDataExt, FromLua, Function, Lua, OwnedFunction, OwnedTable, Table,
    UserData, UserDataMethods, Value,
};
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
    repeat_modifier: Option<Modifier>,
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
            repeat_modifier: None,
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
                if let Some(Modifier::RepeatToFit) = ctx.repeat_modifier {
                    to_update = Some(ctx.last_priority);
                    update_modifier = Some(Modifier::RepeatToFit);
                }
                break;
            }

            if ctx.last_priority == priority && ctx.repeat_modifier == Some(Modifier::RepeatOnce) {
                to_update = Some(ctx.last_priority);
                update_modifier = Some(Modifier::RepeatOnce);
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
            output.operations,
        );

        ctx.screen.get().borrow_mut().update(lua, image).unwrap(); // TODO handle errors

        ctx.repeat_modifier = match (output.repeat_modifier, end_auto_repeat) {
            (Some(Modifier::RepeatToFit), _) => Some(Modifier::RepeatToFit),
            (Some(Modifier::RepeatOnce), false) => Some(Modifier::RepeatOnce),
            (_, _) => None,
        };
        ctx.time_remaining = match update_modifier {
            Some(Modifier::RepeatToFit) => ctx.time_remaining,
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
            screen.repeat_modifier = None;
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

struct UserScript {
    action: OwnedFunction,
    predicate: Option<OwnedFunction>,
    run_on: Vec<String>,
}

impl<'lua> FromLua<'lua> for UserScript {
    fn from_lua(value: Value<'lua>, _lua: &'lua Lua) -> mlua::Result<Self> {
        match value {
            Value::Table(table) => {
                let action: Function = table.get("action")?;
                let predicate: Option<Function> = table.get("predicate")?;
                let run_on = table.get("run_on")?;

                Ok(UserScript {
                    action: action.into_owned(),
                    predicate: predicate.and_then(|p| Some(p.into_owned())),
                    run_on,
                })
            }
            _ => Err(mlua::Error::runtime(
                "User script argument shall be a table",
            )),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Modifier {
    RepeatOnce,
    RepeatToFit,
}

struct ScriptOutput {
    operations: Vec<Operation>,
    duration: Duration,
    repeat_modifier: Option<Modifier>,
}

impl<'lua> FromLua<'lua> for ScriptOutput {
    fn from_lua(value: Value<'lua>, _lua: &'lua Lua) -> mlua::Result<Self> {
        match value {
            Value::Table(table) => {
                let data = table.get("data")?;

                let duration = table
                    .get("duration")
                    .and_then(|duration| Ok(Duration::from_millis(duration)))
                    .unwrap_or(DEFAULT_UPDATE_TIME);

                let repeat_once = table.get("repeat_once").unwrap_or(false);
                let repeat_to_fit = table.get("repeat_to_fit").unwrap_or(false);
                let modifier = match (repeat_once, repeat_to_fit) {
                    (false, false) => None,
                    (true, false) => Some(Modifier::RepeatOnce),
                    (false, true) => Some(Modifier::RepeatToFit),
                    (true, true) => {
                        return Err(mlua::Error::runtime(
                            "repeat_once and repeat_to_fit can't both be set to true",
                        ))
                    }
                };

                Ok(ScriptOutput {
                    operations: data,
                    duration,
                    repeat_modifier: modifier,
                })
            }
            _ => Err(mlua::Error::runtime("Script output shall be a table")),
        }
    }
}
