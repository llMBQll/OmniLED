#![windows_subsystem = "windows"]

use log::error;
use mlua::{AnyUserData, AnyUserDataExt, Lua, Table, TableExt};
use std::sync::atomic::AtomicBool;

use crate::app_loader::app_loader::AppLoader;
use crate::common::common::proto_to_lua_value;
use crate::constants::constants::Constants;
use crate::events::event_loop::EventLoop;
use crate::events::event_queue::{Event, EventQueue};
use crate::events::events::Events;
use crate::events::shortcuts::Shortcuts;
use crate::keyboard::keyboard::{process_events, KeyboardEventEventType};
use crate::logging::logger::Logger;
use crate::renderer::renderer::Renderer;
use crate::screen::screens::Screens;
use crate::script_handler::script_handler::ScriptHandler;
use crate::settings::settings::Settings;
use crate::tray_icon::tray_icon::TrayIcon;

mod app_loader;
mod common;
mod constants;
mod events;
mod keyboard;
mod logging;
mod model;
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

    let _logger = Logger::new(&lua);
    let _shortcuts = Shortcuts::load(&lua);
    let _events = Events::load(&lua);
    Constants::load(&lua);
    Settings::load(&lua);
    EventQueue::load(&lua);
    server::server::load(&lua);
    let _screens = Screens::load(&lua);
    Renderer::load(&lua);
    let _sandbox = ScriptHandler::load(&lua);
    let _tray = TrayIcon::new(&RUNNING);
    let _apps = AppLoader::load(&lua);

    let keyboard_handle = std::thread::spawn(|| process_events(&RUNNING));

    let interval = Settings::get().update_interval;
    let event_loop = EventLoop::new();
    event_loop
        .run(interval, &RUNNING, |events| {
            let event_handler: Table = lua.globals().get("EVENT_HANDLER").unwrap();
            let shortcuts: AnyUserData = lua.globals().get("SHORTCUTS").unwrap();
            let interval = interval.as_millis() as u64;

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

                            // TODO error handling
                            event_handler
                                .call_method::<_, ()>(
                                    "send_value",
                                    (application.clone(), name, value),
                                )
                                .unwrap();
                        }
                    }
                    Event::Keyboard(event) => {
                        let event_name = format!("KEY({})", event.key);
                        let event_type = match event.event_type {
                            KeyboardEventEventType::Press => "Pressed",
                            KeyboardEventEventType::Release => "Released",
                        };

                        shortcuts
                            .call_method::<_, ()>("process_key", (event_name, event_type))
                            .unwrap();
                    }
                }
            }

            // TODO error handling
            event_handler
                .call_method::<_, ()>("update", interval)
                .unwrap();
        })
        .await;

    keyboard_handle.join().unwrap();
}
