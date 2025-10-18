use common::{ApplicationsConfig, DevicesConfig, ScriptsConfig, SettingsConfig};
use mlua::{Lua, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::task::LocalSet;

mod common;

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
            common::get_test_app_path("test_events")
        )),
        DevicesConfig(String::from(r#"-- Empty"#)),
        ScriptsConfig(String::from(
            r#"EVENTS:register('TEST_EVENTS.End', function(event, value) end_test() end)"#,
        )),
        SettingsConfig(String::from(r#"Settings { log_level = 'Debug' }"#)),
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
