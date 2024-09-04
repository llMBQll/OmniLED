use log::warn;
use mlua::{
    chunk, ErrorContext, FromLua, Function, Lua, OwnedFunction, OwnedTable, Table, UserData,
    UserDataMethods, Value,
};
use oled_derive::{FromLuaValue, UserDataIdentifier};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use crate::common::user_data::{UserDataIdentifier, UserDataRef};
use crate::common::{common::exec_file, scoped_value::ScopedValue};
use crate::create_table_with_defaults;
use crate::events::shortcuts::Shortcuts;
use crate::renderer::renderer::{ContextKey, Renderer};
use crate::screen::screen::Screen;
use crate::screen::screens::{ScreenStatus, Screens};
use crate::script_handler::script_data_types::{load_script_data_types, Operation};
use crate::settings::settings::{get_full_path, Settings};

#[derive(UserDataIdentifier)]
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

    fn reset(&mut self, screen_name: &String) {
        match self.screens.iter_mut().find(|x| x.name == *screen_name) {
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

    pub(self) fn register(
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
        let always_fn = lua.create_function(|_, _: ()| Ok(true)).unwrap();

        let never_fn = lua.create_function(|_, _: ()| Ok(false)).unwrap();

        let times_fn = lua
            .create_function(|lua, n: usize| {
                let mut count = 0;
                lua.create_function_mut(move |_, _: ()| {
                    count += 1;
                    Ok(count <= n)
                })
            })
            .unwrap();

        let env = create_table_with_defaults!(lua, {
            EVENTS = EVENTS,
            LOG = LOG,
            PLATFORM = PLATFORM,
            SHORTCUTS = SHORTCUTS,
            PREDICATE = {
                Always = $always_fn,
                Never = $never_fn,
                Times = $times_fn,
            }
        });
        env.set(ScreenBuilder::identifier(), ScreenBuilder).unwrap();
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
            handler.reset(&screen);
            Ok(())
        });
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

#[derive(UserDataIdentifier)]
struct ScreenBuilder;

impl UserData for ScreenBuilder {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("new", |lua, _, name: String| {
            let screens = UserDataRef::<Screens>::load(lua);

            let builder = match screens.get().screen_status(&name) {
                Some(ScreenStatus::Available) => Ok(ScreenBuilderImpl::new(name)),
                Some(ScreenStatus::Loaded) => Err(mlua::Error::RuntimeError(format!(
                    "Device '{}' already loaded.",
                    name
                ))),
                None => Err(mlua::Error::RuntimeError(format!(
                    "Device '{}' not found.",
                    name
                ))),
            };

            builder
        });
    }
}

#[derive(Clone)]
enum BuilderType {
    Screen,
    Script,
}

#[derive(Clone)]
struct ScreenBuilderImpl {
    scripts: Vec<UserScript>,
    shortcut: Vec<String>,
    device_name: String,
    builder_type: Option<BuilderType>,
    screen_count: usize,
    current_screen: Rc<RefCell<usize>>,
}

impl ScreenBuilderImpl {
    pub fn new(name: String) -> Self {
        Self {
            scripts: vec![],
            shortcut: vec![],
            device_name: name,
            builder_type: None,
            screen_count: 0,
            current_screen: Rc::new(RefCell::new(0)),
        }
    }
}

impl UserData for ScreenBuilderImpl {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("with_script", |_lua, builder, script: UserScript| {
            if let Some(BuilderType::Screen) = builder.builder_type {
                return Err(mlua::Error::RuntimeError(
                    "Can't use 'with_script' after calling 'with_screen' or 'with_screen_toggle'."
                        .to_string(),
                ));
            }
            builder.builder_type = Some(BuilderType::Script);

            builder.scripts.push(script);

            Ok(builder.clone())
        });

        methods.add_method_mut("with_screen", |lua, builder, scripts: Vec<UserScript>| {
            if let Some(BuilderType::Script) = builder.builder_type {
                return Err(mlua::Error::RuntimeError(
                    "Can't use 'with_screen' after calling 'with_script'.".to_string(),
                ));
            }
            builder.builder_type = Some(BuilderType::Screen);

            let screen = builder.screen_count;
            builder.screen_count += 1;

            for mut script in scripts {
                let current_screen = builder.current_screen.clone();
                let predicate = script.predicate;
                let wrapper = lua
                    .create_function(move |_, _: ()| {
                        if *current_screen.borrow() != screen {
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

        methods.add_method_mut("with_screen_toggle", |_lua, builder, keys: Vec<String>| {
            if let Some(BuilderType::Script) = builder.builder_type {
                return Err(mlua::Error::RuntimeError(
                    "Can't use 'with_screen_toggle' after calling 'with_script'.".to_string(),
                ));
            }
            builder.builder_type = Some(BuilderType::Screen);

            builder.shortcut = keys;

            Ok(builder.clone())
        });

        methods.add_method_mut("register", |lua, builder, _: ()| {
            if builder.screen_count > 1 && !builder.shortcut.is_empty() {
                let current = builder.current_screen.clone();
                let count = builder.screen_count;
                let name = builder.device_name.clone();
                let toggle_screen = lua
                    .create_function_mut(move |lua, _: ()| {
                        *current.borrow_mut() += 1;
                        if *current.borrow() == count {
                            *current.borrow_mut() = 0;
                        }

                        let mut script_handler = UserDataRef::<ScriptHandler>::load(lua);
                        script_handler.get_mut().reset(&name);

                        Ok(())
                    })
                    .unwrap();

                let mut shortcuts = UserDataRef::<Shortcuts>::load(lua);
                shortcuts
                    .get_mut()
                    .register(builder.shortcut.clone(), toggle_screen)?;
            }

            let mut script_handler = UserDataRef::<ScriptHandler>::load(lua);
            script_handler.get_mut().register(
                lua,
                builder.device_name.clone(),
                builder.scripts.clone(),
            )?;

            Ok(())
        });
    }
}
