#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use log::debug;
use mlua::Lua;
#[cfg(not(target_os = "macos"))]
use omni_led_lib::tray_icon::tray_icon::TrayIcon;
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
};
use std::sync::atomic::AtomicBool;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

mod logging;

static RUNNING: AtomicBool = AtomicBool::new(true);

#[tokio::main]
async fn main() {
    let event_loop = winit::event_loop::EventLoop::<UiEvent>::with_user_event()
        .build()
        .unwrap();

    let scripting_thread = std::thread::spawn(|| {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
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
            Events::load(&lua);
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
                    let dispatcher = UserDataRef::<Events>::load(&lua);
                    for event in events {
                        dispatcher.get().dispatch(&lua, event).unwrap();
                    }

                    let mut script_handler = UserDataRef::<ScriptHandler>::load(&lua);
                    script_handler.get_mut().update(&lua, interval).unwrap();
                })
                .await;

            keyboard_handle.join().unwrap();
        });
    });

    let mut ui_handler = UiHandler;
    event_loop.run_app(&mut ui_handler).unwrap();

    _ = scripting_thread.join();
}

pub enum UiEvent {}

struct UiHandler;

impl ApplicationHandler<UiEvent> for UiHandler {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: UiEvent) {}

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
    }
}
