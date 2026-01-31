use log::error;
use pixels::{Pixels, SurfaceTexture};
use std::collections::HashMap;
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

pub struct Handler {
    event_loop: EventLoop<Event>,
    handler_impl: HandlerImpl,
}

impl Handler {
    pub fn new<F: FnOnce() + 'static>(constants: Constants, on_init: F) -> Self {
        let event_loop = EventLoop::<Event>::with_user_event().build().unwrap();
        let proxy = event_loop.create_proxy();

        PROXY.get_or_init(|| HandlerProxy { proxy });

        Self {
            event_loop,
            handler_impl: HandlerImpl::new(constants, on_init),
        }
    }

    pub fn run(mut self) {
        self.event_loop.run_app(&mut self.handler_impl).unwrap();
    }
}

struct WindowContext {
    window: Arc<Window>,
    pixels: Pixels<'static>,
    window_handle: WindowHandle,
}

struct HandlerImpl {
    windows: HashMap<WindowId, WindowContext>,
    icon: Icon,
    constants: Constants,
    on_init: Option<Box<dyn FnOnce()>>,
    _tray: Option<TrayIcon>,
}

impl HandlerImpl {
    fn new<F: FnOnce() + 'static>(constants: Constants, on_init: F) -> Self {
        Self {
            windows: HashMap::new(),
            icon: window_icon_image(),
            constants,
            on_init: Some(Box::new(on_init)),
            _tray: None,
        }
    }
}

impl ApplicationHandler<Event> for HandlerImpl {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if let StartCause::Init = cause {
            self._tray = Some(TrayIcon::new(
                self.constants.clone(),
                PROXY.get().unwrap().clone(),
            ));

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

                let surface = SurfaceTexture::new(width, height, Arc::clone(&window));
                let pixels = Pixels::new(width, height, surface).unwrap();

                self.windows.insert(
                    window.id(),
                    WindowContext {
                        window,
                        pixels,
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

                let _ = ctx.pixels.resize_surface(new_size.width, new_size.height);
            }
            WindowEvent::RedrawRequested => {
                let ctx = match self.windows.get_mut(&window_id) {
                    Some(ctx) => ctx,
                    None => return,
                };

                ctx.window_handle.draw(ctx.pixels.frame_mut());
                ctx.pixels.render().unwrap();
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
