#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use clap::Parser;
use log::debug;
use mlua::Lua;
use omni_led_lib::{
    common::common::load_internal_functions,
    common::user_data::UserDataRef,
    constants::config::{ConfigType, read_config, write_default_configs},
    constants::constants::Constants,
    devices::devices::Devices,
    events::dispatcher::Dispatcher,
    events::event_loop::EventLoop,
    events::events::Events,
    events::shortcuts::Shortcuts,
    keyboard::keyboard::process_events,
    logging::logger::Log,
    plugin_loader::plugin_loader::PluginLoader,
    script_handler::script_handler::ScriptHandler,
    settings::settings::Settings,
    ui::event::Event,
    ui::handler::{HandlerBuilder, PROXY},
};
use std::sync;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

#[cfg(target_os = "windows")]
mod console;
mod logging;

static RUNNING: AtomicBool = AtomicBool::new(true);

fn main() {
    let _options = match Options::try_parse() {
        Ok(options) => {
            #[cfg(target_os = "windows")]
            if options.attach_console {
                console::attach_console_if_missing();
            }

            options
        }
        Err(err) => {
            #[cfg(target_os = "windows")]
            console::attach_console_if_missing();

            err.exit();
        }
    };

    set_panic_hook();

    let (ready_tx, ready_rx) = sync::mpsc::channel();

    let scripting_thread = std::thread::spawn(move || {
        let init_begin = Instant::now();

        let lua = Lua::new();

        load_internal_functions(&lua);
        Constants::load(&lua);

        let _ = ready_rx.recv().unwrap();

        let log_handle = logging::init();
        Log::load(&lua, log_handle);

        write_default_configs().unwrap();

        let devices_config = read_config(ConfigType::Devices).unwrap();
        let plugins_config = read_config(ConfigType::Plugins).unwrap();
        let scripts_config = read_config(ConfigType::Scripts).unwrap();
        let settings_config = read_config(ConfigType::Settings).unwrap();

        Settings::load(&lua, settings_config);
        let mut dispatcher = Dispatcher::load(&lua);
        Events::load(&lua);
        Shortcuts::load(&lua);
        Devices::load(&lua, devices_config);
        ScriptHandler::load(&lua, scripts_config);
        PluginLoader::load(&lua, plugins_config);

        let init_end = Instant::now();
        debug!("Initialized in {:?}", init_end - init_begin);

        let settings = UserDataRef::<Settings>::load(&lua);
        let interval = settings.get().update_interval;
        let event_loop = EventLoop::new();
        event_loop.run(interval, &RUNNING, |events| {
            for event in events {
                dispatcher.dispatch(&lua, event).unwrap();
            }

            let mut script_handler = UserDataRef::<ScriptHandler>::load(&lua);
            script_handler.get_mut().update(&lua, interval).unwrap();
        });
    });

    let keyboard_thread = std::thread::spawn(|| process_events(&RUNNING));

    HandlerBuilder::new()
        .with_on_init(move || ready_tx.send(true).unwrap())
        .run();

    RUNNING.store(false, Ordering::Relaxed);
    _ = scripting_thread.join();
    _ = keyboard_thread.join().unwrap();
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

#[derive(Parser)]
#[command(name = "OmniLED")]
#[command(version, about)]
struct Options {
    /// Attach console to the program. Applies only on Windows.
    #[clap(short, long, default_value = "false")]
    attach_console: bool,
}
