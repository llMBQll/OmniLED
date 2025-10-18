use common::{ApplicationsConfig, DevicesConfig, ScriptsConfig, SettingsConfig};
use mlua::{Integer, Lua, Number, Value};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::task::LocalSet;

mod common;

static RUNNING: AtomicBool = AtomicBool::new(true);

macro_rules! count {
    () => (0);
    ($head:expr $(, $rest:expr)*) => (1 + count!($($rest),*));
}

macro_rules! match_event_data {
    ($actual:expr, $($expected:expr),+ $(,)?) => {
        {
            let mut index = 0;
            let expected_count = count!($($expected),+);
            assert_eq!($actual.len(), expected_count, "Expected {} events, got {}", expected_count, $actual.len());
            $(
                assert_eq!(
                    EventData::from($actual[index].clone()),
                    $expected,
                    "Mismatch at index {}",
                    index
                );
                index += 1;
            )+
            _ = index;
        }
    };
}

#[tokio::test]
async fn event_handler() {
    let config_dir = PathBuf::from("tests/config");
    common::setup_config(
        &config_dir,
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
    let (_lua, mut events) = local
        .run_until(async move { common::run_omni_led(&RUNNING, config_dir, custom_fns).await })
        .await;

    events.retain(|event| event.0.starts_with("TEST_EVENTS"));
    match_event_data!(
        events,
        EventData {
            name: String::from("TEST_EVENTS"),
            value: EventDataValue::Table(vec![EventData {
                name: String::from("Begin"),
                value: EventDataValue::Integer(0),
            }]),
        },
        EventData {
            name: String::from("TEST_EVENTS.Begin"),
            value: EventDataValue::Integer(0),
        },
        EventData {
            name: String::from("TEST_EVENTS"),
            value: EventDataValue::Table(vec![EventData {
                name: String::from("End"),
                value: EventDataValue::Integer(0),
            }]),
        },
        EventData {
            name: String::from("TEST_EVENTS.End"),
            value: EventDataValue::Integer(0),
        }
    );
}

#[derive(Debug)]
struct EventData {
    name: String,
    value: EventDataValue,
}

impl PartialEq for EventData {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.value == other.value
    }
}

impl From<(String, Value)> for EventData {
    fn from((name, value): (String, Value)) -> Self {
        Self {
            name,
            value: EventDataValue::from(value),
        }
    }
}

#[derive(Debug)]
enum EventDataValue {
    Nil,
    Boolean(bool),
    Integer(Integer),
    Number(Number),
    String(String),
    Table(Vec<EventData>),
    // Any,
}

impl PartialEq for EventDataValue {
    fn eq(&self, other: &Self) -> bool {
        match &self {
            EventDataValue::Nil => match other {
                EventDataValue::Nil => true,
                _ => false,
            },
            EventDataValue::Boolean(val) => match other {
                EventDataValue::Boolean(other_val) => *val == *other_val,
                _ => false,
            },
            EventDataValue::Integer(val) => match other {
                EventDataValue::Integer(other_val) => *val == *other_val,
                _ => false,
            },
            EventDataValue::Number(val) => match other {
                EventDataValue::Number(other_val) => *val == *other_val,
                _ => false,
            },
            EventDataValue::String(val) => match other {
                EventDataValue::String(other_val) => *val == *other_val,
                _ => false,
            },
            EventDataValue::Table(val) => match other {
                EventDataValue::Table(other_val) => *val == *other_val,
                _ => false,
            },
            // EventDataValue::Any => true,
        }
    }
}

impl From<Value> for EventDataValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Nil => EventDataValue::Nil,
            Value::Boolean(val) => EventDataValue::Boolean(val),
            Value::Integer(val) => EventDataValue::Integer(val),
            Value::Number(val) => EventDataValue::Number(val),
            Value::String(val) => EventDataValue::String(val.to_string_lossy()),
            Value::Table(table) => {
                let mut fields = table
                    .pairs::<String, Value>()
                    .map(|pair| {
                        let (key, value) = pair.unwrap();
                        EventData {
                            name: key,
                            value: EventDataValue::from(value),
                        }
                    })
                    .collect::<Vec<_>>();
                fields.sort_by(|a, b| a.name.cmp(&b.name));
                EventDataValue::Table(fields)
            }
            other => panic!("Unexpected value: {:#?}", { other }),
        }
    }
}
