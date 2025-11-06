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

#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use clap::Parser;
use log::debug;
use mlua::Lua;
use omni_led_lib::{
    app_loader::app_loader::AppLoader, common::common::load_internal_functions,
    common::user_data::UserDataRef, constants::configs::Configs, constants::constants::Constants,
    devices::devices::Devices, events::event_loop::EventLoop, events::events::Events,
    events::shortcuts::Shortcuts, keyboard::keyboard::process_events, logging::logger::Log,
    script_handler::script_handler::ScriptHandler, server::server::PluginServer,
    settings::settings::Settings,
};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::time::Instant;
use tray_icon::TrayIcon;

mod logging;
mod tray_icon;

static RUNNING: AtomicBool = AtomicBool::new(true);

#[tokio::main]
async fn main() {
    let init_begin = Instant::now();

    let options = Options::parse();

    let lua = Lua::new();

    load_internal_functions(&lua);
    Constants::load(&lua, options.config_dir);
    Configs::load(&lua);

    let log_handle = logging::init(&lua);
    Log::load(&lua, log_handle);

    Settings::load(&lua);
    PluginServer::load(&lua).await;
    Events::load(&lua);
    Shortcuts::load(&lua);
    Devices::load(&lua);
    ScriptHandler::load(&lua);
    AppLoader::load(&lua);

    let _tray = TrayIcon::new(&lua, &RUNNING);

    let keyboard_handle = std::thread::spawn(|| process_events(&RUNNING));

    let init_end = Instant::now();
    debug!("Initialized in {:?}", init_end - init_begin);

    let settings = UserDataRef::<Settings>::load(&lua);
    let interval = settings.get().update_interval;
    let event_loop = EventLoop::new();
    event_loop
        .run(interval, &RUNNING, |events| {
            let dispatcher = UserDataRef::<Events>::load(&lua);
            for event in events {
                dispatcher.get().dispatch(&lua, event).unwrap();
            }

            let mut script_handler = UserDataRef::<ScriptHandler>::load(&lua);
            script_handler.get_mut().update(&lua, interval).unwrap();
        })
        .await;

    keyboard_handle.join().unwrap();
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Options {
    #[clap(short, long)]
    config_dir: Option<PathBuf>,
}
