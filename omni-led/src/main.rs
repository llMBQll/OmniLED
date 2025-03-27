/*
 * OmniLED is a software for displaying data on various OLED devices.
 * Copyright (C) 2024  Michał Bałabanow <m.balabanow@gmail.com>
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

#![windows_subsystem = "windows"]

use crate::app_loader::app_loader::AppLoader;
use crate::common::common::{load_internal_functions, proto_to_lua_value};
use crate::common::user_data::UserDataRef;
use crate::constants::constants::Constants;
use crate::devices::devices::Devices;
use crate::events::event_loop::EventLoop;
use crate::events::event_queue::Event;
use crate::events::events::Events;
use crate::events::shortcuts::Shortcuts;
use crate::keyboard::keyboard::{KeyboardEventEventType, process_events};
use crate::logging::logger::Log;
use crate::script_handler::script_handler::ScriptHandler;
use crate::server::server::PluginServer;
use crate::settings::settings::Settings;
use crate::tray_icon::tray_icon::TrayIcon;
use log::{debug, error};
use mlua::Lua;
use omni_led_api::types::{Field, field};
use std::sync::atomic::AtomicBool;
use std::time::Instant;

mod app_loader;
mod common;
mod constants;
mod devices;
mod events;
mod keyboard;
mod logging;
mod renderer;
mod script_handler;
mod server;
mod settings;
mod tray_icon;

static RUNNING: AtomicBool = AtomicBool::new(true);

#[tokio::main]
async fn main() {
    let init_begin = Instant::now();

    let lua = Lua::new();

    load_internal_functions(&lua);
    Log::load(&lua);
    Constants::load(&lua);
    Settings::load(&lua);
    PluginServer::load(&lua).await;
    Events::load(&lua);
    Shortcuts::load(&lua);
    Devices::load(&lua);
    ScriptHandler::load(&lua);
    AppLoader::load(&lua);

    let _tray = TrayIcon::new(&RUNNING);

    let keyboard_handle = std::thread::spawn(|| process_events(&RUNNING));

    let init_end = Instant::now();
    debug!("Initialized in {:?}", init_end - init_begin);

    let settings = UserDataRef::<Settings>::load(&lua);
    let interval = settings.get().update_interval;
    let event_loop = EventLoop::new();
    event_loop
        .run(interval, &RUNNING, |events| {
            let mut event_dispatcher = UserDataRef::<Events>::load(&lua);
            let mut shortcuts = UserDataRef::<Shortcuts>::load(&lua);
            let mut script_handler = UserDataRef::<ScriptHandler>::load(&lua);

            for event in events {
                match event {
                    Event::Application((application, value)) => {
                        let value = Field {
                            field: Some(field::Field::FTable(value)),
                        };
                        let value = match proto_to_lua_value(&lua, value) {
                            Ok(value) => value,
                            Err(err) => {
                                error!("Failed to convert protobuf value: {}", err);
                                continue;
                            }
                        };

                        script_handler
                            .get_mut()
                            .set_value(&lua, application, value)
                            .unwrap();
                    }
                    Event::Keyboard(event) => {
                        let key_name = format!("KEY({})", event.key);
                        let action = match event.event_type {
                            KeyboardEventEventType::Press => "Pressed",
                            KeyboardEventEventType::Release => "Released",
                        };

                        shortcuts
                            .get_mut()
                            .process_key(&lua, &key_name, action)
                            .unwrap();
                    }
                }
            }

            event_dispatcher.get_mut().update();
            shortcuts.get_mut().update();
            script_handler.get_mut().update(&lua, interval).unwrap();
        })
        .await;

    keyboard_handle.join().unwrap();
}
