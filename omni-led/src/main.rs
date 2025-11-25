#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use log::debug;
use mlua::Lua;
use omni_led_lib::{
    app_loader::app_loader::AppLoader,
    common::common::load_internal_functions,
    common::user_data::UserDataRef,
    constants::config::{ConfigType, read_config},
    constants::constants::Constants,
    devices::devices::Devices,
    events::event_loop::EventLoop,
    events::events::Events,
    events::shortcuts::Shortcuts,
    keyboard::keyboard::process_events,
    logging::logger::Log,
    script_handler::script_handler::ScriptHandler,
    server::server::PluginServer,
    settings::settings::Settings,
    tray_icon::tray_icon::TrayIcon,
};
use std::sync::atomic::AtomicBool;
use std::time::Instant;

mod logging;

static RUNNING: AtomicBool = AtomicBool::new(true);

#[tokio::main]
async fn main() {
    let init_begin = Instant::now();

    let lua = Lua::new();

    load_internal_functions(&lua);
    Constants::load(&lua);

    let log_handle = logging::init(&lua);
    Log::load(&lua, log_handle);

    let applications_config = read_config(&lua, ConfigType::Applications).unwrap();
    let devices_config = read_config(&lua, ConfigType::Devices).unwrap();
    let scripts_config = read_config(&lua, ConfigType::Scripts).unwrap();
    let settings_config = read_config(&lua, ConfigType::Settings).unwrap();

    Settings::load(&lua, settings_config);
    PluginServer::load(&lua).await;
    let mut dispatcher = Events::load(&lua);
    Shortcuts::load(&lua);
    Devices::load(&lua, devices_config);
    ScriptHandler::load(&lua, scripts_config);
    AppLoader::load(&lua, applications_config);

    let _tray = TrayIcon::new(&lua, &RUNNING);

    let keyboard_handle = std::thread::spawn(|| process_events(&RUNNING));

    let init_end = Instant::now();
    debug!("Initialized in {:?}", init_end - init_begin);

    let settings = UserDataRef::<Settings>::load(&lua);
    let interval = settings.get().update_interval;
    let event_loop = EventLoop::new();
    event_loop
        .run(interval, &RUNNING, |events| {
            for event in events {
                dispatcher.dispatch(&lua, event).unwrap();
            }

            let mut script_handler = UserDataRef::<ScriptHandler>::load(&lua);
            script_handler.get_mut().update(&lua, interval).unwrap();
        })
        .await;

    keyboard_handle.join().unwrap();
}
