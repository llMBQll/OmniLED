use crate::common::user_data::{UserDataIdentifier, UserDataRef};
use crate::common::{common::exec_file, scoped_value::ScopedValue};
use crate::create_table_with_defaults;
use crate::events::shortcuts::Shortcuts;
use crate::renderer::renderer::{ContextKey, Renderer};
use crate::screen::screen::Screen;
use crate::screen::screens::Screens;
use crate::script_handler::script_data_types::{load_script_data_types, Operation};
use crate::settings::settings::{get_full_path, Settings};
use log::warn;
use mlua::{
    chunk, ErrorContext, FromLua, Function, Lua, OwnedFunction, OwnedTable, Table, UserData,
    UserDataMethods, Value,
};
use oled_derive::FromLuaValue;
use std::time::Duration;

pub struct ScriptHandler {
    environment: OwnedTable,
    renderer: Renderer,
    screens: Vec<ScreenContext>,
}

struct ScreenContext {
    screen: Box<dyn Screen>,
    name: String,
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
            Self::identifier(),
            ScriptHandler {
                renderer: Renderer::new(lua),
                environment: environment.clone().into_owned(),
                screens: vec![],
            },
        );

        let settings = UserDataRef::<Settings>::load(lua);
        exec_file(
            lua,
            &get_full_path(&settings.get().scripts_file),
            environment,
        )
        .unwrap();

        ScreenDataMap::load(lua);

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
            let empty = lua.create_table().unwrap();
            env.set(application_name.clone(), empty).unwrap();
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

    fn reset(&mut self, screen_name: String) {
        match self.screens.iter_mut().find(|x| x.name == screen_name) {
            Some(ctx) => {
                ctx.marked_for_update = vec![false; ctx.marked_for_update.len()];
                ctx.time_remaining = Duration::ZERO;
                ctx.last_priority = 0;
                ctx.repeats = None;
            }
            None => {
                warn!("Screen {} not found", screen_name);
            }
        }
    }

    fn register(
        &mut self,
        lua: &Lua,
        screen_name: String,
        user_scripts: Vec<UserScript>,
    ) -> mlua::Result<()> {
        let mut screens = UserDataRef::<Screens>::load(lua);
        let screen = screens.get_mut().load_screen(lua, screen_name.clone())?;

        let screen_count = self.screens.len();
        let script_count = user_scripts.len();

        let context = ScreenContext {
            screen,
            name: screen_name,
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

    fn test_predicate(function: &Option<OwnedFunction>) -> mlua::Result<bool> {
        let predicate = match function {
            Some(predicate) => predicate.call::<_, bool>(())?,
            None => true,
        };
        Ok(predicate)
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

            if marked_for_update && Self::test_predicate(&ctx.scripts[priority].predicate)? {
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
        let memory_representation = ctx.screen.memory_representation(lua)?;
        env.set("SCREEN", size)?;

        let output: ScriptOutput = ctx.scripts[to_update].action.call(())?;
        let (end_auto_repeat, image) = renderer.render(
            ContextKey {
                script: to_update,
                screen: ctx.index,
            },
            size,
            output.data,
            memory_representation,
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
            reset = function(screen)
                SCRIPT_HANDLER:reset(screen)
            end,
            setmetatable = setmetatable,
            EVENTS = EVENTS,
            LOG = LOG,
            PLATFORM = PLATFORM,
            SHORTCUTS = SHORTCUTS,
            PREDICATE = {
                Always = function()
                    return true
                end,
                Never = function()
                    return false
                end,
                Times = function(x)
                    local count = 0
                    return function()
                        if (count >= x) then
                            return false
                        end
                        count = count + 1
                        return true
                    end
                end,
            }
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

        methods.add_method_mut("reset", |_lua, handler, screen: String| {
            handler.reset(screen);
            Ok(())
        });
    }
}

impl UserDataIdentifier for ScriptHandler {
    fn identifier() -> &'static str {
        "SCRIPT_HANDLER"
    }
}

#[derive(FromLuaValue, Clone)]
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

#[derive(FromLuaValue, Debug, PartialEq, Copy, Clone)]
enum Repeat {
    Once,
    ToFit,
}

#[derive(FromLuaValue, Clone)]
struct ScriptOutput {
    data: Vec<Operation>,

    #[mlua(transform(Self::transform_duration))]
    duration: Duration,

    repeats: Option<Repeat>,
}

impl ScriptOutput {
    fn transform_duration(duration: Option<u64>, _lua: &Lua) -> mlua::Result<Duration> {
        let duration = duration
            .and_then(|duration| Some(Duration::from_millis(duration)))
            .unwrap_or(DEFAULT_UPDATE_TIME);
        Ok(duration)
    }
}

#[derive(Clone)]
enum BuilderType {
    Screen,
    Script,
}

#[derive(Clone)]
struct ScreenBuilder {
    scripts: Vec<UserScript>,
    shortcut: Vec<String>,
    device_name: Option<String>,
    builder_type: Option<BuilderType>,
    screen_count: usize,
    id: usize,
}

impl ScreenBuilder {
    pub fn new(lua: &Lua) -> Self {
        let mut data_map = UserDataRef::<ScreenDataMap>::load(lua);
        let id = data_map.get_mut().new_entry();

        Self {
            scripts: vec![],
            shortcut: vec![],
            device_name: None,
            builder_type: None,
            screen_count: 0,
            id,
        }
    }
}

impl UserData for ScreenBuilder {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("with_screen", |lua, builder, scripts: Vec<UserScript>| {
            if let Some(BuilderType::Script) = builder.builder_type {
                // TODO error
            }
            builder.builder_type = Some(BuilderType::Screen);

            let id = builder.id;

            let mut data_map = UserDataRef::<ScreenDataMap>::load(lua);
            data_map.get_mut().add_screen(id);

            for (screen, mut script) in scripts.into_iter().enumerate() {
                let predicate = script.predicate;
                let wrapper = lua
                    .create_function(move |lua, _: ()| {
                        let data_map = UserDataRef::<ScreenDataMap>::load(lua);
                        let current_screen = data_map.get().get_current(id);

                        if screen != current_screen {
                            return Ok(false);
                        }

                        let predicate_check = match &predicate {
                            Some(predicate) => predicate.call::<_, bool>(())?,
                            None => true,
                        };

                        Ok(predicate_check)
                    })
                    .unwrap();

                script.predicate = Some(wrapper.into_owned());
                builder.scripts.push(script);
            }

            Ok(builder.clone())
        });

        methods.add_method_mut("with_script", |_lua, builder, script: UserScript| {
            if let Some(BuilderType::Screen) = builder.builder_type {
                // TODO error
            }
            builder.builder_type = Some(BuilderType::Script);

            builder.scripts.push(script);

            Ok(builder.clone())
        });

        methods.add_method_mut("build", |lua, builder, _: ()| {
            if builder.screen_count > 1 && !builder.shortcut.is_empty() {
                let mut shortcuts = UserDataRef::<Shortcuts>::load(lua);

                let id = builder.id;
                let toggle_screen = lua
                    .create_function(move |lua, _: ()| {
                        let mut data_map = UserDataRef::<ScreenDataMap>::load(lua);
                        data_map.get_mut().toggle(id);
                        Ok(())
                    })
                    .unwrap();

                shortcuts
                    .get_mut()
                    .register(builder.shortcut.clone(), toggle_screen);
            }

            Ok(())
        });
    }
}

#[derive(Clone)]
struct ScreenData {
    current: usize,
    count: usize,
}

impl UserData for ScreenData {}

struct ScreenDataMap {
    data: Vec<ScreenData>,
}

impl ScreenDataMap {
    pub fn load(lua: &Lua) {
        let map = Self { data: vec![] };

        lua.globals().set(Self::identifier(), map).unwrap();
    }

    pub fn new_entry(&mut self) -> usize {
        let id = self.data.len();
        self.data.push(ScreenData {
            current: 0,
            count: 0,
        });
        id
    }

    pub fn add_screen(&mut self, id: usize) {
        self.data[id].count += 1;
    }

    pub fn get_current(&self, id: usize) -> usize {
        self.data[id].current
    }

    pub fn toggle(&mut self, id: usize) {
        self.data[id].current += 1;
        if self.data[id].current == self.data[id].count {
            self.data[id].current = 0;
        }
    }
}

impl UserData for ScreenDataMap {}

impl UserDataIdentifier for ScreenDataMap {
    fn identifier() -> &'static str {
        "SCREEN_DATA_MAP"
    }
}
