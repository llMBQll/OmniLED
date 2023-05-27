use mlua::Lua;

use crate::applications::loader::Loader;
use crate::events::events::Events;
use crate::logging::logger::Logger;
use crate::renderer::renderer::Renderer;
use crate::screen::screens::Screens;
use crate::script_handler::script_handler::ScriptHandler;
use crate::server::server::Server;
use crate::server::update_handler::UpdateHandler;
use crate::settings::settings::Settings;

mod applications;
mod events;
mod logging;
mod model;
mod renderer;
mod script_handler;
mod server;
mod settings;
mod screen;

#[tokio::main]
async fn main() {
    let lua = Lua::new();

    let _logger = Logger::new(&lua);
    Events::load(&lua);
    Settings::load(&lua);
    Server::new(&lua);
    Screens::load(&lua);
    Renderer::load(&lua);

    let _sandbox = ScriptHandler::load(&lua);

    let loader = Loader::new(&lua);
    loader.load().unwrap();
    let runner = UpdateHandler::make_runner(&lua);
    runner.call_async::<_ ,()>(()).await.unwrap();
}