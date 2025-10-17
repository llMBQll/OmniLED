/*
 * OmniLED is a software for displaying data on various OLED devices.
 * Copyright (C) 2025  Michał Bałabanow <m.balabanow@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use mlua::{Lua, Value};
use omni_led_lib::devices::devices::Devices;
use omni_led_lib::{
    app_loader::app_loader::AppLoader, common::common::load_internal_functions,
    common::user_data::UserDataRef, constants::constants::Constants, events::event_loop::EventLoop,
    events::events::Events, events::shortcuts::Shortcuts, logging::logger::Log,
    script_handler::script_handler::ScriptHandler, server::server::PluginServer,
    settings::settings::Settings,
};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;

pub async fn run_omni_led(
    running: &AtomicBool,
    config_dir: PathBuf,
    custom_fns: Vec<(&str, fn(lua: &Lua, value: Value) -> mlua::Result<()>)>,
) -> (Lua, Vec<(String, Value)>) {
    let lua = Lua::new();

    load_internal_functions(&lua);
    Constants::load(&lua, Some(config_dir));
    Log::load(&lua);
    Settings::load(&lua);
    PluginServer::load(&lua).await;
    Events::load(&lua);
    Shortcuts::load(&lua);
    Devices::load(&lua);
    ScriptHandler::load(&lua);
    AppLoader::load(&lua);

    load_custom_functions(&lua, custom_fns);

    let event_vec = Rc::new(RefCell::new(Vec::new()));
    register_event_collector(&lua, event_vec.clone());

    let settings = UserDataRef::<Settings>::load(&lua);
    let interval = settings.get().update_interval;
    let event_loop = EventLoop::new();
    event_loop
        .run(interval, running, |events| {
            let dispatcher = UserDataRef::<Events>::load(&lua);
            for event in events {
                dispatcher.get().dispatch(&lua, event).unwrap();
            }

            let mut script_handler = UserDataRef::<ScriptHandler>::load(&lua);
            script_handler.get_mut().update(&lua, interval).unwrap();
        })
        .await;

    (lua, event_vec.take())
}

fn load_custom_functions(
    lua: &Lua,
    custom_fns: Vec<(&str, fn(lua: &Lua, value: Value) -> mlua::Result<()>)>,
) {
    let mut script_handler = UserDataRef::<ScriptHandler>::load(lua);
    for (name, function) in custom_fns {
        script_handler
            .get_mut()
            .set_value(
                lua,
                name.to_string(),
                Value::Function(lua.create_function(function).unwrap()),
            )
            .unwrap();
    }
}

fn register_event_collector(lua: &Lua, events: Rc<RefCell<Vec<(String, Value)>>>) {
    let collector = move |_: &Lua, event: (String, Value)| -> mlua::Result<()> {
        events.borrow_mut().push(event);
        Ok(())
    };

    let mut events = UserDataRef::<Events>::load(lua);
    events
        .get_mut()
        .register("*".to_string(), lua.create_function_mut(collector).unwrap())
        .unwrap();
}
