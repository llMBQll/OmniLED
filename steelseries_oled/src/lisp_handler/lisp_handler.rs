use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use rust_lisp::model::{FloatType, IntType};
use rust_lisp::prelude::*;
use serde_json::Value as JsonValue;

use crate::lisp_handler::custom_functions::*;

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
        let mut env = self.env.as_ref().borrow_mut();
        env.define(
            Symbol::from(plugin.as_str()),
            Value::HashMap(Rc::new(RefCell::new(HashMap::new()))),
        )
    }

    pub fn update(&mut self, plugin: &String, values: &HashMap<String, JsonValue>) {
        let mut env = self.env.as_ref().borrow_mut();

        for (key, value) in values {
            env.define(
                Symbol::from(format!("{}:{}", plugin, key).as_str()),
                Self::json_to_lisp(value),
            );
        }
    }

    // pub fn _update(&mut self, plugin: &String, values: &HashMap<String, JsonValue>) {
    //     let env  = self.env.as_ref().borrow_mut();
    //     let map = match env.get(&Symbol::from(plugin.as_str())).unwrap() {
    //         Value::HashMap(map) => map,
    //         _ => panic!("HashMap expected")
    //     };
    //     let mut map = map.as_ref().borrow_mut();
    //     for (key, value) in values {
    //         *map
    //             .entry(Value::Symbol(Symbol::from(key.as_str())))
    //             .or_insert(Value::NIL) = Self::json_to_lisp(value);
    //     }
    // }

    pub fn process_handlers(&mut self, handlers: &Vec<Value>) -> Vec<Value> {
        let mut vec = Vec::with_capacity(handlers.len());
        for handler in handlers {
            let x = eval(self.env.clone(), handler).unwrap();
            vec.push(x);
        }
        vec
    }

    fn load_handler_functions(env: &mut Env) {
        env.define(Symbol::from("%"), Value::NativeFunc(modulo));
        // replace division operator to allow mixed int and float division and
        // return an error on division by zero rather than panicking

        env.undefine(&Symbol::from("/"));
        env.define(Symbol::from("/"), Value::NativeFunc(divide));

        env.define(Symbol::from(":"), Value::NativeFunc(from_hashmap));

        env.define(Symbol::from("format"), Value::NativeFunc(format));

        env.define(Symbol::from("bar"), Value::NativeFunc(bar));

        env.define(Symbol::from("text"), Value::NativeFunc(text));
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
            serde_json::Value::String(string) => Value::String(string.to_string()),
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
                    lisp_map.insert(Value::String(key.to_string()), Self::json_to_lisp(value));
                }
                Value::HashMap(Rc::new(RefCell::new(lisp_map)))
            }
        }
    }
}