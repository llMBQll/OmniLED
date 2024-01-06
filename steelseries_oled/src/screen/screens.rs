use mlua::{chunk, Function, LightUserData, Lua, UserData, UserDataMethods, Value};
use std::collections::HashMap;
use std::ffi::c_void;

use crate::common::common::exec_file;
use crate::common::scoped_value::ScopedValue;
use crate::create_table;
use crate::screen::screen::Screen;
use crate::screen::steelseries_engine::steelseries_engine_device::SteelseriesEngineDevice;
use crate::screen::usb_device::usb_device::USBDevice;
use crate::settings::settings::{get_full_path, Settings};

pub struct Screens {
    screens: HashMap<String, ScreenWrapper>,
}

impl Screens {
    pub fn load(lua: &Lua) -> ScopedValue {
        lua.globals()
            .set("SCREEN_INITIALIZERS", lua.create_table().unwrap())
            .unwrap();

        let screens = ScopedValue::new(
            lua,
            "SCREENS",
            Screens {
                screens: HashMap::new(),
            },
        );

        Self::load_screens(lua);

        screens
    }

    pub fn load_screens(lua: &Lua) {
        let load_steelseries_engine_device = Self::make_loader::<SteelseriesEngineDevice>(lua);
        let load_usb_device = Self::make_loader::<USBDevice>(lua);

        let env = create_table!(lua, {
            steelseries_engine_device = $load_steelseries_engine_device,
            usb_device = $load_usb_device,
            PLATFORM = PLATFORM,
            table = { insert = table.insert, maxn = table.maxn, remove = table.remove, sort = table.sort }
        });

        exec_file(
            lua,
            &get_full_path(&Settings::get().supported_screens_file),
            env,
        );
    }

    fn make_loader<T: Screen + 'static>(lua: &Lua) -> Function {
        let loader = |lua: &Lua, settings: Value| -> mlua::Result<()> {
            let initializer = |lua: &Lua, settings: Value| -> mlua::Result<LightUserData> {
                // TODO error handling
                let screen = T::init(lua, settings).unwrap();
                let screen = Box::new(ScreenWrapper::new(screen));
                let ptr = Box::into_raw(screen);
                let data = LightUserData(ptr as *mut c_void);
                Ok(data)
            };
            let initializer = lua.create_function(initializer).unwrap();

            lua.load(chunk! {
                local settings = $settings;
                SCREEN_INITIALIZERS[settings.name] = function() return $initializer(settings) end
            })
            .exec()
            .unwrap();

            Ok(())
        };

        lua.create_function(loader).unwrap()
    }

    fn from_user_data(user_data: LightUserData) -> &'static mut dyn Screen {
        let wrapped = unsafe { &mut *(user_data.0 as *mut ScreenWrapper) };
        wrapped.inner.as_mut()
    }
}

impl UserData for Screens {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("load_screen", |lua, this, key: String| {
            // TODO add error handling
            let screen_name = key.clone();
            let screen = this.screens.entry(key).or_insert_with(|| {
                let user_data: LightUserData = lua
                    .load(chunk! {
                        SCREEN_INITIALIZERS[$screen_name]()
                    })
                    .eval()
                    .unwrap();
                let screen = unsafe { Box::from_raw(user_data.0 as _) };
                *screen
            });
            Ok(LightUserData(screen as *mut _ as *mut c_void))
        });

        methods.add_method(
            "update",
            |lua, _, (wrapped, data): (LightUserData, Vec<u8>)| {
                let screen = Self::from_user_data(wrapped);
                screen.update(lua, data).unwrap(); // TODO map and log error

                Ok(())
            },
        );

        methods.add_method("size", |lua, _, wrapped: LightUserData| {
            let screen = Self::from_user_data(wrapped);
            match screen.size(lua) {
                Ok(size) => Ok(Some(size)),
                Err(_) => Ok(None), // TODO log error
            }
        });
    }
}

struct ScreenWrapper {
    pub inner: Box<dyn Screen>,
}

impl ScreenWrapper {
    pub fn new<T: Screen + 'static>(screen: T) -> Self {
        Self {
            inner: Box::new(screen),
        }
    }
}
