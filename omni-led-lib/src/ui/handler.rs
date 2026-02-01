use log::error;
use softbuffer::{Context, Surface};
use std::collections::HashMap;
use std::num::NonZero;
use std::sync::atomic::Ordering;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::window::{Icon, Window, WindowAttributes, WindowId};

use crate::constants::constants::Constants;
use crate::ui::event::Event;
use crate::ui::icon_image::window_icon_image;
use crate::ui::tray_icon::TrayIcon;
use crate::ui::window::WindowHandle;

pub struct HandlerBuilder {
    event_loop: EventLoop<Event>,
    constants: Option<Constants>,
    on_init: Option<Box<dyn FnOnce()>>,
}

impl HandlerBuilder {
    pub fn new() -> Self {
        let event_loop = EventLoop::<Event>::with_user_event().build().unwrap();
        let proxy = event_loop.create_proxy();

        PROXY.get_or_init(|| HandlerProxy { proxy });

        Self {
            event_loop,
            constants: None,
            on_init: None,
        }
    }

    pub fn with_constants(mut self, constants: Constants) -> Self {
        self.constants = Some(constants);
        self
    }

    pub fn with_on_init<F: FnOnce() + 'static>(mut self, on_init: F) -> Self {
        self.on_init = Some(Box::new(on_init));
        self
    }

    pub fn run(self) {
        #[cfg(target_os = "linux")]
        Self::run_tray_icon_thread(self.constants.clone().unwrap());

        let mut handler = Handler::new(self.constants, self.on_init.unwrap());
        self.event_loop.run_app(&mut handler).unwrap();
    }

    #[cfg(target_os = "linux")]
    fn run_tray_icon_thread(constants: Constants) {
        std::thread::spawn(move || {
            gtk::init().unwrap();
            let _tray = Some(TrayIcon::new(constants, PROXY.get().unwrap().clone()));
            gtk::main();
        });
    }
}

struct WindowContext {
    window: Arc<Window>,
    surface: Surface<Arc<Window>, Arc<Window>>,
    window_handle: WindowHandle,
}

struct Handler {
    windows: HashMap<WindowId, WindowContext>,
    icon: Icon,
    #[cfg_attr(target_os = "linux", allow(unused))]
    constants: Option<Constants>,
    on_init: Option<Box<dyn FnOnce()>>,
    _tray: Option<TrayIcon>,
}

impl Handler {
    fn new(constants: Option<Constants>, on_init: Box<dyn FnOnce()>) -> Self {
        Self {
            windows: HashMap::new(),
            icon: window_icon_image(),
            constants,
            on_init: Some(on_init),
            _tray: None,
        }
    }
}

impl ApplicationHandler<Event> for Handler {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if let StartCause::Init = cause {
            #[cfg(not(target_os = "linux"))]
            {
                self._tray = Some(TrayIcon::new(
                    self.constants.take().unwrap(),
                    PROXY.get().unwrap().clone(),
                ));
            }

            self.on_init.take().unwrap()();
        }
    }

    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: Event) {
        match event {
            Event::OpenWindow(window_handle) => {
                let width = window_handle.size.width as u32;
                let height = window_handle.size.height as u32;
                let size = LogicalSize::new(width, height);

                let attributes = WindowAttributes::default()
                    .with_title(window_handle.name.clone())
                    .with_window_icon(Some(self.icon.clone()))
                    .with_resizable(true)
                    .with_inner_size(size)
                    .with_min_inner_size(size);

                let window = match event_loop.create_window(attributes) {
                    Ok(window) => Arc::new(window),
                    Err(err) => {
                        error!("{}", err);
                        event_loop.exit();
                        return;
                    }
                };

                window_handle
                    .id
                    .store(window.id().into(), Ordering::Release);

                let context = Context::new(Arc::clone(&window)).unwrap();
                let mut surface = Surface::new(&context, Arc::clone(&window)).unwrap();
                surface
                    .resize(NonZero::new(width).unwrap(), NonZero::new(height).unwrap())
                    .unwrap();

                self.windows.insert(
                    window.id(),
                    WindowContext {
                        window,
                        surface,
                        window_handle,
                    },
                );
            }
            Event::UpdateWindow(id) => {
                if let Some(ctx) = self.windows.get(&id.into()) {
                    ctx.window.request_redraw();
                }
            }
            Event::Quit => event_loop.exit(),
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                self.windows.remove(&window_id);
            }
            WindowEvent::Resized(new_size) => {
                let ctx = match self.windows.get_mut(&window_id) {
                    Some(ctx) => ctx,
                    None => return,
                };

                if let (Some(width), Some(height)) =
                    (NonZero::new(new_size.width), NonZero::new(new_size.height))
                {
                    ctx.surface.resize(width, height).unwrap();
                }
            }
            WindowEvent::RedrawRequested => {
                let ctx = match self.windows.get_mut(&window_id) {
                    Some(ctx) => ctx,
                    None => return,
                };

                let mut buffer = ctx.surface.buffer_mut().unwrap();
                let width = buffer.width().get() as usize;
                let height = buffer.height().get() as usize;
                ctx.window_handle.draw(&mut *buffer, width, height);
                buffer.present().unwrap();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Force winit to wake up after the tray icon puts it to sleep.
        // Without this winit ignores all screen update events after the tray icon is shown.

        let wait_until = Instant::now() + Duration::from_millis(16);
        event_loop.set_control_flow(ControlFlow::WaitUntil(wait_until));

        for ctx in self.windows.values() {
            ctx.window.request_redraw();
        }
    }
}

pub static PROXY: OnceLock<HandlerProxy> = OnceLock::new();

#[derive(Clone)]
pub struct HandlerProxy {
    proxy: EventLoopProxy<Event>,
}

impl HandlerProxy {
    pub fn send(&self, event: Event) {
        if let Err(err) = self.proxy.send_event(event) {
            error!("Failed to send event: {}", err);
        }
    }
}
