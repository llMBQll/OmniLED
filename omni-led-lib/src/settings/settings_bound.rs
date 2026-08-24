use mlua::{AnyUserData, FromLua, Lua, ObjectLike, Value};
use std::{cell::RefCell, rc::Rc};

use crate::common::lua_traits::LuaName;
use crate::events::event_handle::EventHandle;
use crate::events::events::Events;
use crate::script_handler::script_data_types::EventKey;
use crate::settings::settings::Settings;

struct SettingsBound<T> {
    value: Rc<RefCell<T>>,
    handle: EventHandle,
}

impl<T: FromLua + Clone + 'static> SettingsBound<T> {
    pub fn new(lua: &Lua, setting: &'static str) -> Self {
        let value = Rc::new(RefCell::new(Self::read_initial(lua, setting)));
        let handle = Self::register_handler(lua, setting, Rc::clone(&value));

        Self { value, handle }
    }

    pub fn get(&self) -> T {
        self.value.borrow().clone()
    }

    fn read_initial(lua: &Lua, setting: &'static str) -> T {
        if setting.contains(".") {
            todo!("Add handling for nested settings, currently there are none")
        }
        let x: AnyUserData = lua.globals().get(Settings::NAME).unwrap();
        x.get(setting).unwrap()
    }

    fn register_handler(lua: &Lua, setting: &'static str, value: Rc<RefCell<T>>) -> EventHandle {
        let key = EventKey::String(format!("Settings.{setting}"));

        let assign_fn = lua
            .create_function_mut(move |_, (_, new_value): (Value, T)| {
                *value.borrow_mut() = new_value;
                Ok(())
            })
            .unwrap();

        Events::register(key, assign_fn, true)
    }
}

impl<T> Drop for SettingsBound<T> {
    fn drop(&mut self) {
        Events::unregister(self.handle.clone());
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;

    use mlua::IntoLua;

    use crate::common::user_data::set_unique_user_data;
    use crate::events::dispatcher::Dispatcher;
    use crate::events::event_queue::{Event, EventQueue};

    fn env() -> (Lua, Dispatcher) {
        let lua = Lua::new();
        let settings = Settings::default();
        set_unique_user_data(&lua, settings);
        let dispatcher = Dispatcher::load(&lua);
        (lua, dispatcher)
    }

    fn update_settings(lua: &Lua, field: &'static str, value: impl IntoLua) {
        let settings: AnyUserData = lua.globals().get(Settings::NAME).unwrap();
        settings.set(field, value).unwrap();
    }

    fn process_events(lua: &Lua, dispatcher: &mut Dispatcher) {
        let events = EventQueue::instance().lock().unwrap().get_events();
        println!("{:#?}", events);
        for event in events {
            dispatcher.dispatch(lua, event).unwrap();
        }
    }

    #[test]
    fn initial_load() {
        let (lua, _dispatcher) = env();
        let animation_ticks_delay = SettingsBound::<usize>::new(&lua, "animation_ticks_delay");
        assert_eq!(
            animation_ticks_delay.get(),
            Settings::default().animation_ticks_delay
        );
    }

    #[test]
    fn handle_update() {
        let (lua, mut dispatcher) = env();
        let animation_ticks_delay = SettingsBound::<usize>::new(&lua, "animation_ticks_delay");

        const NEW_VALUE: usize = 700;
        assert_ne!(NEW_VALUE, animation_ticks_delay.get()); // just in case assert the value will actually change

        update_settings(&lua, "animation_ticks_delay", NEW_VALUE);
        process_events(&lua, &mut dispatcher);

        assert_eq!(NEW_VALUE, animation_ticks_delay.get());
    }

    #[test]
    fn unregister() {
        let (lua, mut dispatcher) = env();
        let animation_ticks_delay = SettingsBound::<usize>::new(&lua, "animation_ticks_delay");

        process_events(&lua, &mut dispatcher);

        drop(animation_ticks_delay);

        let events = EventQueue::instance().lock().unwrap().get_events();
        assert_eq!(events.len(), 1);
        assert_matches!(events[0], Event::Unregister(_));
    }
}
