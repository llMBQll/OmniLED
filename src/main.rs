use std::collections::HashMap;

use std::sync::{Arc, Mutex};
use mlua::Lua;


use tokio::time::{Duration, Instant};
use warp::Filter;
use warp::sse::Event;
use crate::applications::loader::Loader;

// use crate::ApplicationMetadataReplyData::{Reason, Token};
use crate::applications::applications::{Applications, load_applications};
use crate::events::events::Events;
use crate::keyboard_api::KeyboardAPI;
use crate::logging::logger::Logger;
// use crate::model::display::Display;
// use crate::model::operation::Operation;
// use crate::plugin::plugin::Plugin;
use crate::renderer::renderer::Renderer;
use crate::script_handler::script_handler::ScriptHandler;
use crate::server::server::Server;
use crate::server::update_handler::UpdateHandler;
use crate::settings::settings::Settings;

mod applications;
mod events;
mod keyboard_api;
mod logging;
mod model;
mod plugins;
mod renderer;
mod script_handler;
mod server;
mod settings;

#[tokio::main]
async fn main() {
    let lua = Lua::new();

    let _logger = Logger::new(&lua);
    Events::load(&lua);
    Settings::load(&lua);
    Server::new(&lua);

    let _sandbox = ScriptHandler::load(&lua);

    let loader = Loader::new(&lua);
    loader.load_applications().unwrap();
    let runner = UpdateHandler::make_runner(&lua);
    runner.call_async::<_ ,()>(()).await.unwrap();
}


// async fn main() {
//     let renderer = setup_renderer();
//     let keyboard_api = setup_keyboard_api();
//
//     run_server(renderer, keyboard_api).await;
// }

fn _setup_keyboard_api() -> KeyboardAPI {
    KeyboardAPI::new()
}

fn _setup_renderer() -> Renderer {
    // TODO: allow to change screen size dynamically
    // TODO: expose screen dimensions as lisp environment variables

    const WIDTH: usize = 128;
    const HEIGHT: usize = 40;

    Renderer::new(HEIGHT, WIDTH)
}

async fn _run_server(_renderer: Renderer, _keyboard_api: KeyboardAPI) {
    tokio::time::sleep(Duration::MAX).await;
}