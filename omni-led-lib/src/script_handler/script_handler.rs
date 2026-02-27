use log::warn;
use mlua::{ErrorContext, FromLua, Function, Lua, Table, UserData, UserDataMethods, Value, chunk};
use omni_led_api::plugin::Plugin;
use omni_led_derive::{FromLuaValue, UniqueUserData};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use crate::common::user_data::{UniqueUserData, UserDataRef};
use crate::constants::config::{ConfigType, load_config};
use crate::create_table_with_defaults;
use crate::devices::device::Device;
use crate::devices::devices::{DeviceStatus, Devices};
use crate::events::events::{Events, get_cleanup_entries_metatable};
use crate::events::shortcuts::Shortcuts;
use crate::renderer::animation::State;
use crate::renderer::animation_group::AnimationGroup;
use crate::renderer::renderer::Renderer;
use crate::script_handler::script_data_types::Widget;

#[derive(UniqueUserData)]
pub struct ScriptHandler {
    environment: Table,
    renderer: Renderer,
    devices: Vec<DeviceContext>,
}

struct DeviceContext {
    device: Box<dyn Device>,
    name: String,
    layouts: Vec<Layout>,
    animation_groups: Vec<HashMap<usize, AnimationGroup>>,
    layout_update_flags: Vec<bool>,
    time_remaining: Duration,
    last_priority: usize,
    state: State,
}

const DEFAULT_UPDATE_TIME: Duration = Duration::from_millis(1000);

impl ScriptHandler {
    pub fn load(lua: &Lua, config: String) {
        let environment = Self::make_sandbox(lua);

        Self::set_unique(
            lua,
            ScriptHandler {
                renderer: Renderer::new(lua),
                environment: environment.clone(),
                devices: vec![],
            },
        );

        let event_handler = lua
            .create_function(|lua, (event, value): (String, Value)| {
                let mut this = UserDataRef::<ScriptHandler>::load(lua);
                this.get_mut().mark_for_update(&event);

                if Plugin::is_valid_identifier(&event) {
                    // Set values recursively only from top-level application events
                    this.get().set_value(lua, event, value)?;
                }

                Ok(())
            })
            .unwrap();

        let mut events = UserDataRef::<Events>::load(lua);
        events
            .get_mut()
            .register("*".to_string(), event_handler)
            .unwrap();

        load_config(lua, ConfigType::Scripts, &config, environment).unwrap();
    }

    pub fn set_value(&self, lua: &Lua, value_name: String, value: Value) -> mlua::Result<()> {
        let env = &self.environment;
        Self::set_value_impl(lua, env, &value_name, value)
    }

    fn set_value_impl(
        lua: &Lua,
        parent: &Table,
        value_name: &str,
        value: Value,
    ) -> mlua::Result<()> {
        match value {
            Value::Table(table) => match get_cleanup_entries_metatable(&table)? {
                Some(cleanup_entries) => {
                    if !parent.contains_key(value_name)? {
                        let empty = lua.create_table()?;
                        parent.set(value_name, empty)?;
                    }
                    let entry: Table = parent.get(value_name)?;

                    table.for_each(|key: String, val: Value| {
                        Self::set_value_impl(lua, &entry, &key, val)
                    })?;

                    cleanup_entries.for_each(|key: String, _: Value| {
                        Self::set_value_impl(lua, &entry, &key, Value::Nil)
                    })
                }
                None => {
                    // This path is for tables that are just arrays
                    parent.set(value_name, Value::Table(table))
                }
            },
            value => parent.set(value_name, value),
        }
    }

    fn mark_for_update(&mut self, key: &String) {
        for device in &mut self.devices {
            for (index, layout) in device.layouts.iter().enumerate() {
                if layout.run_on.contains(key) {
                    device.layout_update_flags[index] = true;
                }
            }
        }
    }

    pub fn update(&mut self, lua: &Lua, time_passed: Duration) -> mlua::Result<()> {
        let env = &self.environment;
        for device in &mut self.devices {
            Self::update_impl(lua, device, &mut self.renderer, &env, time_passed)?;
        }
        Ok(())
    }

    fn reset(&mut self, device_name: &String) {
        match self.devices.iter_mut().find(|x| x.name == *device_name) {
            Some(ctx) => {
                ctx.layout_update_flags.fill(false);
                ctx.time_remaining = Duration::ZERO;
                ctx.last_priority = 0;
                ctx.state = State::Finished;
            }
            None => {
                warn!("Device {} not found", device_name);
            }
        }
    }

    fn register(
        &mut self,
        lua: &Lua,
        device_name: String,
        layouts: Vec<Layout>,
    ) -> mlua::Result<()> {
        let mut devices = UserDataRef::<Devices>::load(lua);
        let device = devices.get_mut().load_device(lua, device_name.clone())?;

        let layout_count = layouts.len();

        let context = DeviceContext {
            device,
            name: device_name,
            layouts,
            animation_groups: vec![HashMap::new(); layout_count],
            layout_update_flags: vec![false; layout_count],
            time_remaining: Default::default(),
            last_priority: 0,
            state: State::Finished,
        };
        self.devices.push(context);

        Ok(())
    }

    fn test_predicate(function: &Option<Function>) -> mlua::Result<bool> {
        let predicate = match function {
            Some(predicate) => predicate.call::<_>(())?,
            None => true,
        };
        Ok(predicate)
    }

    fn update_impl(
        lua: &Lua,
        ctx: &mut DeviceContext,
        renderer: &mut Renderer,
        env: &Table,
        time_passed: Duration,
    ) -> mlua::Result<()> {
        ctx.time_remaining = ctx.time_remaining.saturating_sub(time_passed);
        let has_time_remaining = !ctx.time_remaining.is_zero();

        let mut to_update = None;
        let mut new_update = false;
        for (priority, marked_for_update) in ctx.layout_update_flags.iter().enumerate() {
            // If a more important layout still has time remaining, don't bother checking further
            if has_time_remaining && ctx.last_priority < priority {
                break;
            }

            if *marked_for_update && Self::test_predicate(&ctx.layouts[priority].predicate)? {
                to_update = Some(priority);
                new_update = true;
                break;
            }

            // Handle repetition if currently processed priority is equal to that of the last update
            if ctx.last_priority == priority {
                // For `Repeat::ForDuration` make sure that there is still time remaining
                let repeat_for_duration = has_time_remaining && ctx.state == State::CanFinish;

                // For `Repeat::Once` make sure that the animation is in progress
                let repeat_once = ctx.state == State::InProgress;

                if repeat_for_duration || repeat_once {
                    to_update = Some(priority);
                    break;
                }
            }
        }

        ctx.layout_update_flags.fill(false);

        let to_update = match to_update {
            Some(to_update) => to_update,
            None => return Ok(()),
        };
        let screen_changed = to_update != ctx.last_priority;

        let size = ctx.device.size(lua)?;
        let memory_layout = ctx.device.memory_layout(lua)?;
        env.set("SCREEN", size)?;

        let output: LayoutData = ctx.layouts[to_update].layout.call(())?;
        let (animation_state, image) = renderer.render(
            &mut ctx.animation_groups[to_update],
            screen_changed,
            size,
            output.widgets,
            memory_layout,
        );

        ctx.device.update(lua, image)?;

        if new_update {
            ctx.time_remaining = output.duration;
        }
        ctx.last_priority = to_update;
        ctx.state = animation_state;

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

        env
    }
}

impl UserData for ScriptHandler {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut(
            "register",
            |lua, handler, (device, layouts): (String, Vec<Layout>)| {
                handler.register(lua, device, layouts)
            },
        );

        methods.add_method_mut("reset", |_lua, handler, device: String| {
            handler.reset(&device);
            Ok(())
        });
    }
}

#[derive(FromLuaValue, Clone)]
struct Layout {
    layout: Function,
    predicate: Option<Function>,
    run_on: Vec<String>,
}

#[derive(FromLuaValue, Clone)]
struct LayoutData {
    widgets: Vec<Widget>,

    #[mlua(transform = Self::transform_duration)]
    duration: Duration,
}

impl LayoutData {
    fn transform_duration(duration: Option<u64>, _lua: &Lua) -> mlua::Result<Duration> {
        let duration = duration
            .and_then(|duration| Some(Duration::from_millis(duration)))
            .unwrap_or(DEFAULT_UPDATE_TIME);
        Ok(duration)
    }
}

#[derive(UniqueUserData)]
struct ScreenBuilder;

impl UserData for ScreenBuilder {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("new", |lua, _, name: String| {
            let devices = UserDataRef::<Devices>::load(lua);

            let builder = match devices.get().device_status(&name) {
                Some(DeviceStatus::Available) => Ok(ScreenBuilderImpl::new(name)),
                Some(DeviceStatus::Loaded) => Err(mlua::Error::RuntimeError(format!(
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
    Layout,
    LayoutGroup,
}

#[derive(Clone)]
struct ScreenBuilderImpl {
    layouts: Vec<Layout>,
    shortcut: Vec<String>,
    device_name: String,
    builder_type: Option<BuilderType>,
    screen_count: usize,
    current_screen: Rc<RefCell<usize>>,
}

impl ScreenBuilderImpl {
    pub fn new(name: String) -> Self {
        Self {
            layouts: vec![],
            shortcut: vec![],
            device_name: name,
            builder_type: None,
            screen_count: 0,
            current_screen: Rc::new(RefCell::new(0)),
        }
    }
}

impl UserData for ScreenBuilderImpl {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("with_layout", |_lua, builder, layout: Layout| {
            if let Some(BuilderType::LayoutGroup) = builder.builder_type {
                return Err(mlua::Error::RuntimeError(
                    "Can't use 'with_layout' after calling 'with_layout_group' or 'with_layout_group_toggle'."
                        .to_string(),
                ));
            }
            builder.builder_type = Some(BuilderType::Layout);

            builder.layouts.push(layout);

            Ok(builder.clone())
        });

        methods.add_method_mut("with_layout_group", |lua, builder, layouts: Vec<Layout>| {
            if let Some(BuilderType::Layout) = builder.builder_type {
                return Err(mlua::Error::RuntimeError(
                    "Can't use 'with_layout_group' after calling 'with_layout'.".to_string(),
                ));
            }
            builder.builder_type = Some(BuilderType::LayoutGroup);

            let screen = builder.screen_count;
            builder.screen_count += 1;

            if layouts.len() == 0 {
                warn!(
                    "Registering a layout group for device '{}' with 0 layouts",
                    builder.device_name
                );
            }

            for mut layout in layouts {
                let current_screen = builder.current_screen.clone();
                let predicate = layout.predicate;
                let wrapper = lua
                    .create_function(move |_, _: ()| {
                        if *current_screen.borrow() != screen {
                            return Ok(false);
                        }

                        let predicate_check = match &predicate {
                            Some(predicate) => predicate.call(())?,
                            None => true,
                        };

                        Ok(predicate_check)
                    })
                    .unwrap();

                layout.predicate = Some(wrapper);
                builder.layouts.push(layout);
            }

            Ok(builder.clone())
        });

        methods.add_method_mut(
            "with_layout_group_toggle",
            |_lua, builder, keys: Vec<String>| {
                if let Some(BuilderType::Layout) = builder.builder_type {
                    return Err(mlua::Error::RuntimeError(
                        "Can't use 'with_layout_group_toggle' after calling 'with_layout'."
                            .to_string(),
                    ));
                }
                builder.builder_type = Some(BuilderType::LayoutGroup);

                builder.shortcut = keys;

                Ok(builder.clone())
            },
        );

        methods.add_method_mut("register", |lua, builder, _: ()| {
            if !builder.shortcut.is_empty() {
                if builder.screen_count < 2 {
                    warn!("Registering shortcut to toggle screens for device '{}', but its screen count is {}", builder.device_name, builder.screen_count);
                }

                let current = builder.current_screen.clone();
                let count = builder.screen_count;
                let name = builder.device_name.clone();
                let toggle_screen = lua
                    .create_function_mut(move |lua, _: ()| {
                        *current.borrow_mut() += 1;
                        if *current.borrow() >= count {
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
                    .register(lua, builder.shortcut.clone(), toggle_screen)?;
            }

            if builder.screen_count == 0 {
                warn!(
                    "Registering device '{}' with zero screens provided",
                    builder.device_name
                );
            }

            let mut script_handler = UserDataRef::<ScriptHandler>::load(lua);
            script_handler.get_mut().register(
                lua,
                builder.device_name.clone(),
                builder.layouts.clone(),
            )?;

            Ok(())
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_table;
    use crate::events::events::proto_to_lua_value;
    use omni_led_derive::IntoProto;

    fn assert_tables_equal(lua: &Lua, left: &Table, right: &Table, line: u32) {
        assert_tables_equal_impl(lua, left, right, &format!("{line}"))
    }

    fn assert_tables_equal_impl(lua: &Lua, left: &Table, right: &Table, path: &str) {
        let left_count = left.pairs::<Value, Value>().count();
        let right_count = right.pairs::<Value, Value>().count();
        assert_eq!(
            left_count, right_count,
            "Tables at '{}' have different sizes: {} vs {}",
            path, left_count, right_count
        );

        for pair in left.pairs::<String, Value>() {
            let (key, left_value) = pair.unwrap();
            let new_path = format!("{}.{}", path, key);
            let right_value: Value = right.raw_get(key).unwrap();

            assert_values_equal(lua, &left_value, &right_value, &new_path);
        }
    }

    fn assert_values_equal(lua: &Lua, left: &Value, right: &Value, path: &str) {
        match (left, right) {
            (Value::Nil, Value::Nil) => {}
            (Value::Boolean(l), Value::Boolean(r)) => {
                assert_eq!(l, r, "Booleans at '{}' differ: {} vs {}", path, l, r);
            }
            (Value::Integer(l), Value::Integer(r)) => {
                assert_eq!(l, r, "Integers at '{}' differ: {} vs {}", path, l, r);
            }
            (Value::Number(l), Value::Number(r)) => {
                assert!(
                    (l - r).abs() < f64::EPSILON,
                    "Numbers at '{}' differ: {} vs {}",
                    path,
                    l,
                    r
                );
            }
            (Value::String(l), Value::String(r)) => {
                assert_eq!(
                    l.to_str().unwrap(),
                    r.to_str().unwrap(),
                    "Strings at '{}' differ: {:?} vs {:?}",
                    path,
                    l.to_str().unwrap(),
                    r.to_str().unwrap()
                );
            }
            (Value::Table(l), Value::Table(r)) => assert_tables_equal_impl(lua, l, r, path),
            (l, r) => {
                // Types don't match, or we have unhandled types, good enough for testing purposes
                panic!(
                    "Unexpected value at '{}': {}({:?}) vs {}({:?})",
                    path,
                    l.type_name(),
                    l,
                    r.type_name(),
                    r
                );
            }
        }
    }

    #[derive(IntoProto)]
    struct InputA {
        a: Option<i64>,
        b: Option<InputB>,
    }

    #[derive(IntoProto)]
    struct InputB {
        b: Option<i64>,
        c: Option<InputC>,
    }

    #[derive(IntoProto)]
    struct InputC {
        c: Option<i64>,
    }

    #[derive(IntoProto)]
    struct InputStrongA {
        a: Option<i64>,
        #[proto(strong_none)]
        b: Option<InputB>,
    }

    #[test]
    fn recursive_set() {
        let lua = Lua::new();
        let env = lua.create_table().unwrap();

        let input = InputA {
            a: Some(1),
            b: Some(InputB {
                b: Some(2),
                c: Some(InputC { c: None }),
            }),
        };
        let input = proto_to_lua_value(&lua, input.into()).unwrap();
        ScriptHandler::set_value_impl(&lua, &env, "a", input).unwrap();

        let expected = create_table! {
            lua,
            {
                a = {
                    a = 1,
                    b = {
                        b = 2,
                        c = { }
                    }
                }
            }
        };
        assert_tables_equal(&lua, &expected, &env, line!());
    }

    #[test]
    fn partial_update() {
        let lua = Lua::new();
        let env = lua.create_table().unwrap();

        let input = InputA {
            a: None,
            b: Some(InputB {
                b: Some(2),
                c: None,
            }),
        };
        let input = proto_to_lua_value(&lua, input.into()).unwrap();
        ScriptHandler::set_value_impl(&lua, &env, "a", input).unwrap();

        let expected = create_table! {
            lua,
            {
                a = {
                    b = {
                        b = 2,
                    }
                }
            }
        };
        assert_tables_equal(&lua, &expected, &env, line!());

        let input = InputA {
            a: Some(1),
            b: Some(InputB {
                b: None,
                c: Some(InputC { c: Some(3) }),
            }),
        };
        let input = proto_to_lua_value(&lua, input.into()).unwrap();
        ScriptHandler::set_value_impl(&lua, &env, "a", input).unwrap();

        let expected = create_table! {
            lua,
            {
                a = {
                    a = 1,
                    b = {
                        b = 2,
                        c = {
                            c = 3,
                        },
                    }
                }
            }
        };
        assert_tables_equal(&lua, &expected, &env, line!());
    }

    #[test]
    fn partial_update_remove() {
        let lua = Lua::new();
        let env = lua.create_table().unwrap();

        let input = InputA {
            a: Some(1),
            b: Some(InputB {
                b: Some(2),
                c: Some(InputC { c: Some(3) }),
            }),
        };
        let input = proto_to_lua_value(&lua, input.into()).unwrap();
        ScriptHandler::set_value_impl(&lua, &env, "a", input).unwrap();

        let expected = create_table! {
            lua,
            {
                a = {
                    a = 1,
                    b = {
                        b = 2,
                        c = {
                            c = 3,
                        }
                    }
                }
            }
        };
        assert_tables_equal(&lua, &expected, &env, line!());

        let input = InputStrongA {
            a: Some(1),
            b: None,
        };
        let input = proto_to_lua_value(&lua, input.into()).unwrap();
        ScriptHandler::set_value_impl(&lua, &env, "a", input).unwrap();

        let expected = create_table! {
            lua,
            {
                a = {
                    a = 1,
                }
            }
        };
        assert_tables_equal(&lua, &expected, &env, line!());
    }
}
