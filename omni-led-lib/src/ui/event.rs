use crate::ui::window::WindowHandle;

pub enum Event {
    OpenWindow(WindowHandle),
    UpdateWindow(u64),
    Quit,
}
