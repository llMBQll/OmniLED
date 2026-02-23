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
    ui::event::Event,
    ui::handler::{HandlerBuilder, PROXY},
};
use std::sync;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

mod logging;

static RUNNING: AtomicBool = AtomicBool::new(true);

fn main() {
    set_panic_hook();

    let (constants_tx, constants_rx) = sync::mpsc::channel();
    let (ready_tx, ready_rx) = sync::mpsc::channel();

    let scripting_thread = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let init_begin = Instant::now();

            let lua = Lua::new();

            load_internal_functions(&lua);
            Constants::load(&lua);

            constants_tx
                .send(UserDataRef::<Constants>::load(&lua).get().clone())
                .unwrap();

            let _ = ready_rx.recv().unwrap();

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

    HandlerBuilder::new()
        .with_constants(constants_rx.recv().unwrap())
        .with_on_init(move || ready_tx.send(true).unwrap())
        .run();

    RUNNING.store(false, Ordering::Relaxed);
    _ = scripting_thread.join();
}

fn set_panic_hook() {
    // Exit all loops on panic to avoid deadlocks on cleanup

    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Some(proxy) = PROXY.get() {
            proxy.send(Event::Quit)
        }
        RUNNING.store(false, Ordering::Relaxed);
        hook(info);
    }));
}
