#![windows_subsystem = "windows"]

use log::error;
use mlua::{AnyUserData, Lua};
use std::sync::atomic::AtomicBool;

use crate::app_loader::app_loader::AppLoader;
use crate::common::common::proto_to_lua_value;
use crate::constants::constants::Constants;
use crate::events::event_loop::EventLoop;
use crate::events::event_queue::Event;
use crate::events::shortcuts::Shortcuts;
use crate::keyboard::keyboard::{process_events, KeyboardEventEventType};
use crate::logging::logger::Logger;
use crate::screen::screens::Screens;
use crate::script_handler::script_handler::ScriptHandler;
use crate::server::server::PluginServer;
use crate::settings::settings::Settings;
use crate::tray_icon::tray_icon::TrayIcon;

mod app_loader;
mod common;
mod constants;
mod events;
mod keyboard;
mod logging;
mod renderer;
mod screen;
mod script_handler;
mod server;
mod settings;
mod tray_icon;

static RUNNING: AtomicBool = AtomicBool::new(true);

#[tokio::main]
async fn main() {
    let lua = Lua::new();

    Logger::load(&lua);
    let _shortcuts = Shortcuts::load(&lua);
    Constants::load(&lua);
    Settings::load(&lua);
    PluginServer::load(&lua);
    let _screens = Screens::load(&lua);
    let _sandbox = ScriptHandler::load(&lua);
    let _tray = TrayIcon::new(&RUNNING);
    let _apps = AppLoader::load(&lua);

    let keyboard_handle = std::thread::spawn(|| process_events(&RUNNING));

    let interval = Settings::get().update_interval;
    let event_loop = EventLoop::new();
    event_loop
        .run(interval, &RUNNING, |events| {
            let script_handler: AnyUserData = lua.globals().get("SCRIPT_HANDLER").unwrap();
            let shortcuts: AnyUserData = lua.globals().get("SHORTCUTS").unwrap();
            let mut shortcuts = shortcuts.borrow_mut::<Shortcuts>().unwrap();

            for event in events {
                match event {
                    Event::Application(event) => {
                        let (application, values) = event;

                        for (name, value) in values {
                            let value = match proto_to_lua_value(&lua, value) {
                                Ok(value) => value,
                                Err(err) => {
                                    error!("Failed to convert json value: {}", err);
                                    continue;
                                }
                            };

                            let mut script_handler =
                                script_handler.borrow_mut::<ScriptHandler>().unwrap();
                            script_handler
                                .set_value(&lua, application.clone(), name, value)
                                .unwrap();
                        }
                    }
                    Event::Keyboard(event) => {
                        let key_name = format!("KEY({})", event.key);
                        let action = match event.event_type {
                            KeyboardEventEventType::Press => "Pressed",
                            KeyboardEventEventType::Release => "Released",
                        };

                        shortcuts.process_key(&lua, &key_name, action).unwrap();
                    }
                }
            }

            let mut script_handler = script_handler.borrow_mut::<ScriptHandler>().unwrap();
            script_handler.update(&lua, interval).unwrap();
        })
        .await;

    keyboard_handle.join().unwrap();
}
