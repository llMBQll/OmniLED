use common::{ApplicationsConfig, DevicesConfig, ScriptsConfig, SettingsConfig};
use mlua::{Lua, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use test_binary::build_test_binary_once;
use tokio::task::LocalSet;

mod common;

build_test_binary_once!(test_events, "testbins");

fn test_events_app_path() -> String {
    let path = path_to_test_events().to_str().unwrap().to_string();
    path.replace("\\", "\\\\")
}

static RUNNING: AtomicBool = AtomicBool::new(true);

#[tokio::test]
async fn event_handler() {
    common::setup_config(
        ApplicationsConfig(format!(
            r#"
            load_app {{
                path = '{}',
                args = {{ '--address', SERVER.Address }},
            }}
            "#,
            test_events_app_path()
        )),
        DevicesConfig(String::from(r#"-- Empty"#)),
        ScriptsConfig(String::from(
            r#"EVENTS:register('TEST_EVENTS.End', function(event, value) end_test() end)"#,
        )),
        SettingsConfig(String::from(r#"-- Use default settings"#)),
    );

    let custom_fns: Vec<(&str, fn(lua: &Lua, value: Value) -> mlua::Result<()>)> =
        vec![("end_test", |_: &Lua, _: Value| {
            RUNNING.store(false, Ordering::Relaxed);
            Ok(())
        })];

    let local = LocalSet::new();
    let (_lua, events) = local
        .run_until(async move { common::run_omni_led(&RUNNING, custom_fns).await })
        .await;

    println!("{:#?}", events);

    assert!(true);
}
