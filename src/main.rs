use mlua::Lua;
use std::sync::atomic::AtomicBool;

use crate::app_loader::app_loader::AppLoader;
use crate::events::events::Events;
use crate::logging::logger::Logger;
use crate::renderer::renderer::Renderer;
use crate::screen::screens::Screens;
use crate::script_handler::script_handler::ScriptHandler;
use crate::server::server::Server;
use crate::server::update_handler::UpdateHandler;
use crate::settings::settings::Settings;
use crate::tray_icon::tray_icon::TrayIcon;

mod app_loader;
mod events;
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

    // TODO find a better place for this
    let table = lua.create_table().unwrap();
    table.set("os", os()).unwrap();
    lua.globals().set("PLATFORM", table).unwrap();

    let _logger = Logger::new(&lua);
    Events::load(&lua);
    Settings::load(&lua);
    Server::new(&lua);
    Screens::load(&lua);
    Renderer::load(&lua);

    let _sandbox = ScriptHandler::load(&lua);

    let _tray = TrayIcon::new(&RUNNING);

    let loader = AppLoader::new(&lua);
    loader.load().unwrap();
    let runner = UpdateHandler::make_runner(&lua, &RUNNING);
    runner.call_async::<_, ()>(()).await.unwrap();
}

// TODO find a better place for this
fn os() -> String {
    #[cfg(target_os = "windows")]
    return String::from("windows");

    #[cfg(target_os = "linux")]
    return String::from("linux");
}