use mlua::{Lua, OwnedFunction};

pub struct CleanupGuard {
    cleanup: OwnedFunction,
}

impl CleanupGuard {
    pub fn with_name(lua: &Lua, name: &str) -> Self {
        let cleanup = lua
            .load(format!("{} = nil", name))
            .into_function()
            .unwrap()
            .into_owned();

        Self { cleanup }
    }
}

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        self.cleanup.call::<_, ()>(()).unwrap()
    }
}
