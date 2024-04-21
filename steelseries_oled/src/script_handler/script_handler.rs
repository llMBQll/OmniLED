use mlua::{
    chunk, AnyUserData, ErrorContext, FromLua, Function, Lua, LuaSerdeExt, OwnedFunction,
    OwnedTable, Table, UserData, UserDataMethods, Value,
};
use oled_derive::FromLuaTable;
use std::time::Duration;

use crate::common::{common::exec_file, scoped_value::ScopedValue};
use crate::create_table_with_defaults;
use crate::renderer::renderer::{ContextKey, Renderer};
use crate::screen::screen::Screen;
use crate::screen::screens::Screens;
use crate::script_handler::script_data_types::{load_script_data_types, Operation};
use crate::settings::settings::{get_full_path, Settings};

pub struct ScriptHandler {
    environment: OwnedTable,
    renderer: Renderer,
    screens: Vec<ScreenContext>,
}

struct ScreenContext {
    screen: Box<dyn Screen>,
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
        )
        .unwrap();

        value
    }

    pub fn set_value(
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

    pub fn update(&mut self, lua: &Lua, time_passed: Duration) -> mlua::Result<()> {
        let env = self.environment.to_ref();
        for screen in &mut self.screens {
            Self::update_impl(lua, screen, &mut self.renderer, &env, time_passed)?;
        }
        Ok(())
    }

    pub fn reset(&mut self) {
        for screen in &mut self.screens {
            screen.marked_for_update = vec![false; screen.marked_for_update.len()];
            screen.time_remaining = Duration::ZERO;
            screen.last_priority = 0;
            screen.repeats = None;
        }
    }

    fn register(
        &mut self,
        lua: &Lua,
        screen_name: String,
        user_scripts: Vec<UserScript>,
    ) -> mlua::Result<()> {
        let screens: AnyUserData = lua.globals().get("SCREENS").unwrap();
        let mut screens = screens.borrow_mut::<Screens>().unwrap();
        let screen = screens.load_screen(lua, screen_name)?;

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
                Some(predicate) => predicate.call::<_, bool>(())?,
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

        let size = ctx.screen.size(lua)?;
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

        ctx.screen.update(lua, image)?;

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

    fn make_sandbox(lua: &Lua) -> Table {
        let env = create_table_with_defaults!(lua, {
            register = function(screen, user_scripts)
                SCRIPT_HANDLER:register(screen, user_scripts)
            end,
            EVENTS = EVENTS,
            LOG = LOG,
            PLATFORM = PLATFORM,
            SHORTCUTS = SHORTCUTS,
        });
        load_script_data_types(lua, &env);

        env
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
