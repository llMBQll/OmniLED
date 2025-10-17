use mlua::{Lua, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::task::LocalSet;

mod common;

static RUNNING: AtomicBool = AtomicBool::new(true);

#[tokio::test]
async fn it_works() {
    let custom_fns: Vec<(&str, fn(lua: &Lua, value: Value) -> mlua::Result<()>)> =
        vec![("end_test", |_: &Lua, _: Value| -> mlua::Result<()> {
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
