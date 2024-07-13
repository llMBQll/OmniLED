#![windows_subsystem = "windows"]

use log::error;
use mlua::Lua;
use std::sync::atomic::AtomicBool;

use crate::app_loader::app_loader::AppLoader;
use crate::common::common::proto_to_lua_value;
use crate::common::user_data::UserDataRef;
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
    PluginServer::load(&lua).await;
    let _screens = Screens::load(&lua);
    let _sandbox = ScriptHandler::load(&lua);
    let _tray = TrayIcon::new(&RUNNING);
    let _apps = AppLoader::load(&lua);

    let keyboard_handle = std::thread::spawn(|| process_events(&RUNNING));

    let settings = UserDataRef::<Settings>::load(&lua);
    let interval = settings.get().update_interval;
    let event_loop = EventLoop::new();
    event_loop
        .run(interval, &RUNNING, |events| {
            let mut shortcuts = UserDataRef::<Shortcuts>::load(&lua);
            let mut script_handler = UserDataRef::<ScriptHandler>::load(&lua);

            for event in events {
                match event {
                    Event::Application(event) => {
                        let (application, values) = event;

                        for (name, value) in values {
                            let value = match proto_to_lua_value(&lua, value) {
                                Ok(value) => value,
                                Err(err) => {
                                    error!("Failed to convert protobuf value: {}", err);
                                    continue;
                                }
                            };

                            script_handler
                                .get_mut()
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

                        shortcuts
                            .get_mut()
                            .process_key(&lua, &key_name, action)
                            .unwrap();
                    }
                }
            }

            script_handler.get_mut().update(&lua, interval).unwrap();
        })
        .await;

    keyboard_handle.join().unwrap();
}
