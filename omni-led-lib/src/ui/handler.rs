use log::error;
use std::sync::OnceLock;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::WindowId;

use crate::constants::constants::Constants;
use crate::ui::event::Event;
use crate::ui::tray_icon::TrayIcon;

pub struct Handler {
    event_loop: EventLoop<Event>,
    handler_impl: HandlerImpl,
    constants: Constants,
}

impl Handler {
    pub fn new(constants: Constants) -> Self {
        let event_loop = EventLoop::<Event>::with_user_event().build().unwrap();
        let proxy = event_loop.create_proxy();

        PROXY.get_or_init(|| HandlerProxy { proxy });

        Self {
            event_loop,
            handler_impl: HandlerImpl,
            constants,
        }
    }

    pub fn run(mut self) {
        let _tray = TrayIcon::new(
            self.constants.clone(),
            HandlerProxy {
                proxy: self.event_loop.create_proxy(),
            },
        );

        self.event_loop.run_app(&mut self.handler_impl).unwrap();
    }
}

struct HandlerImpl;

impl ApplicationHandler<Event> for HandlerImpl {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: Event) {
        match event {
            // Event::Fn(mut f) => f(),
            Event::Quit => event_loop.exit(),
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
    }
}

static PROXY: OnceLock<HandlerProxy> = OnceLock::new();

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

    // pub fn run_on_main_thread<F: FnMut() + Send + 'static>(&self, f: F) {
    //     self.send(Event::Fn(Box::new(f)));
    // }
}
