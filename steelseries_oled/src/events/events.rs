use log::trace;
use mlua::{
    Function, Lua, MetaMethod, OwnedFunction, Table, TableExt, UserData, UserDataMethods, Value,
};

use crate::common::scoped_value::ScopedValue;
use crate::events::key_combination_handler::KeyCombinationHandler;

pub struct Events {
    filter: Option<OwnedFunction>,
    listeners: Vec<OwnedFunction>,
}

impl Events {
    pub fn load(lua: &Lua) -> ScopedValue {
        KeyCombinationHandler::load(lua);

        ScopedValue::new(
            lua,
            "EVENTS",
            Events {
                filter: None,
                listeners: Vec::new(),
            },
        )
    }
}

impl UserData for Events {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("set_filter", |_, this, filter: Function| {
            trace!("Setting filter {:?}", filter);

            this.filter = Some(filter.into_owned());
            Ok(())
        });

        methods.add_method_mut("add_listener", |_, this, filter: Function| {
            trace!("Registering {:?}", filter);

            this.listeners.push(filter.into_owned());
            Ok(())
        });

        methods.add_method("reset_state", |lua, _, _: ()| {
            trace!("Resetting EVENT_HANDLER state");

            let event_handler: Table = lua.globals().get("EVENT_HANDLER")?;
            event_handler.call_method::<_, ()>("reset", ())?;

            Ok(())
        });

        methods.add_method(
            "make_prefixed",
            |_lua, _this, (prefix, event): (String, String)| {
                return Ok(format!("{}({})", prefix, event));
            },
        );

        methods.add_meta_method(
            MetaMethod::Call,
            |_, this, (event, data): (Value, Value)| {
                let event = match &this.filter {
                    Some(filter) => filter.call::<_, Option<Value>>((event, data.clone()))?,
                    None => Some(event),
                };

                match event {
                    Some(event) => {
                        trace!("Emitting {:?}({:?})", event, data);

                        for listener in &this.listeners {
                            listener.call::<_, ()>((event.clone(), data.clone()))?;
                        }
                    }
                    None => {
                        trace!("Filtered {:?}({:?})", event, data);
                    }
                }

                Ok(())
            },
        )
    }
}
