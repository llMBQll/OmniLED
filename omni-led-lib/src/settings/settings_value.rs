use mlua::Lua;

use crate::common::user_data::UserDataRef;
use crate::settings::settings::Settings;

type Accessor<T> = fn(&Settings) -> T;

#[derive(Clone)]
pub struct SettingsValue<T> {
    settings: UserDataRef<Settings>,
    accessor: Accessor<T>,
}

impl<T> SettingsValue<T> {
    pub fn new(lua: &Lua, accessor: Accessor<T>) -> Self {
        let settings = UserDataRef::load(lua);
        Self { settings, accessor }
    }

    pub fn get(&self) -> T {
        let settings = self.settings.get();
        (self.accessor)(&settings)
    }
}
