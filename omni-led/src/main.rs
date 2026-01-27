#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use log::{debug, error};
use mlua::Lua;
use omni_led_lib::devices::emulator::emulator::EmulatorHandle;
use omni_led_lib::{
    OmniLedEvent,
    app_loader::app_loader::AppLoader,
    common::common::{load_internal_functions, set_proxy},
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
    tray_icon::tray_icon::{TrayEvent, TrayIcon},
};
use pixels::{Pixels, SurfaceTexture};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

mod logging;

static RUNNING: AtomicBool = AtomicBool::new(true);

fn main() {
    let event_loop = winit::event_loop::EventLoop::<OmniLedEvent>::with_user_event()
        .build()
        .unwrap();
    let proxy = event_loop.create_proxy();
    let lua_proxy = proxy.clone();

    let handle = std::thread::spawn(move || {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let init_begin = Instant::now();

                let lua = Lua::new();

                load_internal_functions(&lua);
                set_proxy(&lua, lua_proxy);
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
            })
    });

    // TODO move handling to winit event loop
    let keyboard_handle = std::thread::spawn(|| process_events(&RUNNING));

    let _tray = TrayIcon::new(proxy);

    let mut app = App {
        windows: HashMap::new(),
    };
    event_loop.run_app(&mut app).unwrap();

    _ = keyboard_handle.join();
    _ = handle.join();
}

struct App {
    windows: HashMap<WindowId, WindowHandle>,
}

struct WindowHandle {
    window: Arc<Window>,
    pixels: Pixels<'static>,
    emulator_handle: EmulatorHandle,
}

impl App {
    fn exit(event_loop: &ActiveEventLoop) {
        RUNNING.store(false, Ordering::Relaxed);
        event_loop.exit();
    }
}

impl ApplicationHandler<OmniLedEvent> for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: OmniLedEvent) {
        match event {
            OmniLedEvent::NewScreen(emulator_handle) => {
                let width = emulator_handle.width as u32;
                let height = emulator_handle.height as u32;

                let attributes = WindowAttributes::default()
                    .with_title(emulator_handle.name.clone())
                    .with_resizable(true)
                    .with_min_inner_size(LogicalSize::new(width, height));

                let window = match event_loop.create_window(attributes) {
                    Ok(window) => Arc::new(window),
                    Err(err) => {
                        error!("{}", err);
                        Self::exit(&event_loop);
                        return;
                    }
                };

                let surface = SurfaceTexture::new(width, height, Arc::clone(&window));
                let pixels = Pixels::new(width, height, surface).unwrap();

                self.windows.insert(
                    window.id(),
                    WindowHandle {
                        window,
                        pixels,
                        emulator_handle,
                    },
                );
            }
            OmniLedEvent::Tray(e) => match e {
                TrayEvent::Config => {}
                TrayEvent::License => {}
                TrayEvent::Quit => {
                    Self::exit(&event_loop);
                }
            },
            OmniLedEvent::Update => {
                // TODO update only specific windows
                for handle in self.windows.values() {
                    handle.window.request_redraw();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        macro_rules! get_handle_or_return {
            () => {
                match self.windows.get_mut(&window_id) {
                    Some(handle) => handle,
                    None => return,
                }
            };
        }

        match event {
            WindowEvent::CloseRequested => {
                self.windows.remove(&window_id);
            }
            WindowEvent::RedrawRequested => {
                let handle = get_handle_or_return!();

                handle.emulator_handle.draw(handle.pixels.frame_mut());
                handle.pixels.render().unwrap();
            }
            _ => {}
        }
    }
}
