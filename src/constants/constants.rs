use mlua::{chunk, Lua};

pub struct Constants;

impl Constants {
    pub fn load(lua: &Lua) {
        let os = Self::get_os();

        lua.load(chunk! {
            PLATFORM = { os = $os }
        })
        .exec()
        .unwrap();
    }

    fn get_os() -> &'static str {
        #[cfg(target_os = "windows")]
        return "windows";

        #[cfg(target_os = "linux")]
        return "linux";
    }
}
