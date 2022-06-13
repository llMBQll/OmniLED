use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use rust_lisp::model::{FloatType, IntType};
use rust_lisp::prelude::*;
use rust_lisp::utils::{require_parameter, require_string_parameter};
use serde_json::Value as JsonValue;
use strfmt::strfmt;

macro_rules! string {
    ($a:expr) => {
        {
            Value::String(String::from($a))
        }
    }
}

macro_rules! list {
    ($($x:expr),+ $(,)?) => {
        {
            let vec = vec![$($x),+].into_iter();
            Value::List(List::from_iter(vec.into_iter()))
        }
    }
}

pub struct LispHandler {
    env: Rc<RefCell<Env>>,
}

impl LispHandler {
    pub fn new() -> Self {
        let mut env = default_env();
        Self::load_handler_functions(&mut env);
        Self {
            env: Rc::new(RefCell::new(env))
        }
    }

    pub fn register(&mut self, plugin: &String) {
        let mut env  = self.env.as_ref().borrow_mut();
        env.define(
            Symbol::from(plugin.as_str()),
            Value::HashMap(Rc::new(RefCell::new(HashMap::new())))
        )
    }

    pub fn update(&mut self, plugin: &String, values: &HashMap<String, JsonValue>) {
        let env  = self.env.as_ref().borrow_mut();
        let map = match env.get(&Symbol::from(plugin.as_str())).unwrap() {
            Value::HashMap(map) => map,
            _ => panic!("HashMap expected")
        };
        let mut map = map.as_ref().borrow_mut();
        for (key, value) in values {
            *map
                .entry(Value::Symbol(Symbol::from(key.as_str())))
                .or_insert(Value::NIL) = Self::json_to_lisp(value);
        }
    }

    pub fn process_handlers(&mut self, handlers: &Vec<Value>) -> Vec<Value> {
        let mut vec = Vec::with_capacity(handlers.len());
        for handler in handlers {
            let x = eval(self.env.clone(), handler).unwrap();
            vec.push(x);
        }
        vec
    }

    fn load_handler_functions(env: &mut Env) {
        env.define(
            Symbol::from(":"),
            Value::NativeFunc(
                |env, args| {
                    let string = require_string_parameter(":", args, 0)?;
                    let path = string.split(":");

                    let mut iter = path.into_iter();
                    let top_level = iter.next().unwrap();
                    let env = env.as_ref().borrow();
                    let mut val = env.get(&Symbol::from(top_level)).unwrap();
                    for name in iter {
                        match val {
                            Value::List(list) => {
                                let n: usize = name.parse().unwrap();
                                val = list.into_iter().nth(n).unwrap();
                            }
                            Value::HashMap(map) => {
                                let map = map.as_ref().borrow();
                                val = map.get(&Value::Symbol(Symbol::from(name))).unwrap().clone();
                            }
                            _ => { return Err(RuntimeError { msg: String::from("Expected list or HashMap") }); }
                        }
                    }

                    Ok(val)
                }
            )
        );

        env.define(
            Symbol::from("text"),
            Value::NativeFunc(
                |_env, args| {
                    let text = require_parameter("text", args, 0)?;
                    let str = match text {
                        Value::True => "true".to_string(),
                        Value::False => "false".to_string(),
                        Value::Int(int) => int.to_string(),
                        Value::Float(float) => float.to_string(),
                        Value::String(string) => string.to_string(),
                        _ => {
                            return Err(RuntimeError {
                                msg: String::from("Function \"text\" requires argument to be a boolean, a number or a string"),
                            });
                        }
                    };
                    Ok(list![string!("text"), string!(str)])
                }
            ),
        );

        env.define(
            Symbol::from("bar"),
            Value::NativeFunc(
                |_env, args| {
                    let bar = require_parameter("bar", args, 0)?;

                    if let Ok(bar) = TryInto::<FloatType>::try_into(bar) {
                        return Ok(Value::Float(bar));
                    }
                    if let Ok(bar) = TryInto::<IntType>::try_into(bar) {
                        return Ok(Value::Float(bar as FloatType));
                    }

                    Err(RuntimeError {
                        msg: String::from("Function \"bar\" requires argument to be a number"),
                    })
                }
            ),
        );

        env.define(
            Symbol::from("format"),
            Value::NativeFunc(
                |_env, args| {
                    let format = require_string_parameter("format", args, 0)?;

                    let mut vars = HashMap::new();
                    let mut n = 0;
                    let mut iter = args.into_iter();
                    iter.next();
                    for arg in iter {
                        vars.insert(n.to_string(), Self::value_to_string(arg));
                        n += 1;
                    }

                    Ok(string!(strfmt(&format, &vars).unwrap()))
                }
            )
        );
    }

    fn json_to_lisp(json: &JsonValue) -> Value {
        match json {
            serde_json::Value::Null => Value::NIL,
            serde_json::Value::Bool(val) => {
                match val {
                    true => Value::True,
                    false => Value::False,
                }
            }
            serde_json::Value::Number(number) => {
                if let Some(number) = number.as_i64() {
                    return Value::Int(number as IntType);
                }
                if let Some(number) = number.as_f64() {
                    return Value::Float(number as FloatType);
                }
                todo!("{}", "Number Error")
            }
            serde_json::Value::String(string) => string!(string),
            serde_json::Value::Array(array) => {
                let mut lisp_array: Vec<Value> = Vec::new();
                for value in array {
                    lisp_array.push(Self::json_to_lisp(value));
                }
                Value::List(List::from_iter(lisp_array.into_iter()))
            }
            serde_json::Value::Object(map) => {
                let mut lisp_map: HashMap<Value, Value> = HashMap::new();
                for (key, value) in map {
                    lisp_map.insert(string!(key), Self::json_to_lisp(value));
                }
                Value::HashMap(Rc::new(RefCell::new(lisp_map)))
            }
        }
    }

    fn value_to_string(value: &Value) -> String {
        match value {
            Value::String(string) => string.to_string(),
            _ => value.to_string()
        }
    }
}