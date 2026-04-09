use crate::ui::window::WindowHandle;

pub enum Event {
    OpenWindow(WindowHandle),
    UpdateWindow(u64),
    CloseWindow(u64),
    Quit,
}
