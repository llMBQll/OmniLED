use common::{ApplicationsConfig, DevicesConfig, ScriptsConfig, SettingsConfig};
use mlua::{Lua, Value};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::task::LocalSet;

mod common;

static RUNNING: AtomicBool = AtomicBool::new(true);

#[tokio::test]
async fn it_works() {
    let config_dir = PathBuf::from("tests/config");
    let custom_fns: Vec<(&str, fn(lua: &Lua, value: Value) -> mlua::Result<()>)> =
        vec![("end_test", |_: &Lua, _: Value| -> mlua::Result<()> {
            RUNNING.store(false, Ordering::Relaxed);
            Ok(())
        })];

    common::setup_config(
        &config_dir,
        ApplicationsConfig(String::from(
            r#"
            local function get_app_path(name)
                return '..' .. PLATFORM.PathSeparator .. 'target' .. PLATFORM.PathSeparator .. 'debug'
                            .. PLATFORM.PathSeparator .. name .. PLATFORM.ExeSuffix
            end

            load_app {
                path = get_app_path('clock'),
                args = { '--address', SERVER.Address },
            }
            "#,
        )),
        DevicesConfig(String::from(r#"-- Empty"#)),
        ScriptsConfig(String::from(
            r#"
            EVENTS:register('*', function(event, value)
                if event == 'OMNILED.Update' and value == 10 then
                    end_test()
                end
            end)
            "#,
        )),
        SettingsConfig(String::from(
            r#"
            -- Use default settings
            Settings{}
            "#,
        )),
    );

    let local = LocalSet::new();
    let (_lua, events) = local
        .run_until(async move { common::run_omni_led(&RUNNING, config_dir, custom_fns).await })
        .await;

    println!("{:#?}", events);

    assert!(true);
}
