use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::time::Duration;

use rust_lisp::{
    default_env,
    model::{Env, FloatType, IntType, List, RuntimeError, Symbol, Value},
    interpreter::eval,
    parser::parse,
};

use serde_json::Value as JsonValue;

use crate::cast;
use crate::lisp_handler::custom_functions::*;
use crate::model::display::Display;
use crate::model::operation::{Bar, Operation, ScrollingText, Text};
use crate::model::position::Position;

pub struct LispHandler {
    global_env: Rc<RefCell<Env>>,
    local_envs: HashMap<String, Rc<RefCell<Env>>>,
    handlers: HashMap<String, Vec<(Value, Position)>>,
    sensitivity_lists: Vec<(String, HashSet<String>)>,
    last_priority: i64,
    time_remaining: Duration,
}

impl LispHandler {
    pub fn new() -> Self {
        let mut env = default_env();
        Self::load_handler_functions(&mut env);
        Self {
            global_env: Rc::new(RefCell::new(env)),
            local_envs: HashMap::new(),
            handlers: HashMap::new(),
            sensitivity_lists: Vec::new(),
            last_priority: 0,
            time_remaining: Duration::ZERO,
        }
    }

    pub fn register(&mut self, displays: Vec<Display>) -> Result<(), RuntimeError> {
        self.local_envs = HashMap::new();
        self.handlers = HashMap::new();
        self.sensitivity_lists = Vec::new();

        for display in displays {
            self.register_single(display)?;
        }
        Ok(())
    }

    pub fn update(&mut self, plugins: &Vec<(String, HashMap<String, JsonValue>)>, interval: Duration) -> Result<Vec<Operation>, RuntimeError> {
        let mut changed = Vec::<String>::new();

        for (name, values) in plugins {
            for (key, value) in values {
                let symbol_name = format!("{}:{}", name, key);
                self.global_env.as_ref().borrow_mut().define(
                    Symbol::from(symbol_name.as_str()),
                    Self::json_to_lisp(&value),
                );
                changed.push(symbol_name)
            }
        }

        self.time_remaining = self.time_remaining.saturating_sub(interval);
        let mut priority = 0;
        for (name, list) in &self.sensitivity_lists {
            if self.last_priority < priority && !self.time_remaining.is_zero() {
                return Ok(Vec::new())
            }
            for value in &changed {
                if list.contains(value) {
                    self.last_priority = priority;
                    self.time_remaining = Duration::from_millis(2_000);
                    return self.process_handlers(&name);
                }
            }
            priority += 1;
        }

        Ok(Vec::new())
    }

    fn register_single(&mut self, display: Display) -> Result<(), RuntimeError> {
        let mut local = Env::extend(self.global_env.clone());
        local.define(Symbol::from(HASH_MAP_KEY), Value::HashMap(Rc::new(RefCell::new(HashMap::new()))));
        self.local_envs.insert(display.name.to_string(), Rc::new(RefCell::new(local)));

        let mut handlers = Vec::new();
        for (code, pos) in display.parts {
            let mut ast = parse(code.as_str());
            let res = match ast.next() {
                Some(res) => res,
                None => {
                    // allow empty handlers as space fillers
                    return Ok(());
                }
            };
            let handler = match res {
                Ok(handler) => handler,
                Err(error) => {
                    return Err(RuntimeError { msg: format!("Error while parsing the handler: {}", error.msg) });
                }
            };
            handlers.push((handler, pos));
        }

        self.handlers.insert(display.name.to_string(), handlers);
        self.sensitivity_lists.push((display.name, display.sensitivity_list));

        Ok(())
    }

    fn process_handlers(&self, name: &String) -> Result<Vec<Operation>, RuntimeError> {
        let env = self.local_envs.get(name).unwrap();
        let local = env.clone();

        let mut vec = Vec::new();
        let mut hash = 0;
        for (handler, position) in self.handlers.get(name).unwrap() {
            local.as_ref().borrow_mut().define(Symbol::from(CURRENT_INDEX_KEY), Value::Int(hash));
            let x = eval(env.clone(), handler)?;
            vec.push(Self::value_to_operation(&x, position)?);
            hash += 1;
        }
        Ok(vec)
    }

    fn value_to_operation(value: &Value, position: &Position) -> Result<Operation, RuntimeError> {
        // TODO handle errors
        // TODO better operation constructors
        let list = cast!(value, Value::List);
        let mut iter = list.into_iter();
        let op = cast!(iter.next().unwrap(), Value::String);
        match op.as_str() {
            "bar" => {
                let value = cast!(iter.next().unwrap(), Value::Float);
                Ok(Operation::Bar(Bar { value, position: position.clone() }))
            }
            "text" => {
                let text = cast!(iter.next().unwrap(), Value::String);
                Ok(Operation::Text(Text { text, strict: false, upper: false, position: position.clone() }))
            }
            "text-strict" => {
                let text = cast!(iter.next().unwrap(), Value::String);
                Ok(Operation::Text(Text { text, strict: true, upper: false, position: position.clone() }))
            }
            "text-upper" => {
                let text = cast!(iter.next().unwrap(), Value::String);
                // let text = text.to_uppercase();
                Ok(Operation::Text(Text { text, strict: true, upper: true, position: position.clone() }))
            }
            "scrolling-text" => {
                let text = cast!(iter.next().unwrap(), Value::String);
                let count = cast!(iter.next().unwrap(), Value::Int);
                Ok(Operation::ScrollingText(ScrollingText { text, count, position: position.clone() }))
            }
            _ => Err(RuntimeError { msg: "Unknown operation".to_string() })
        }
    }

    fn load_handler_functions(env: &mut Env) {
        // replace math operators to allow mixed int and float division and
        // return an error on division by zero rather than panicking

        env.define(Symbol::from("+"), Value::NativeFunc(add));
        env.define(Symbol::from("-"), Value::NativeFunc(subtract));
        env.define(Symbol::from("*"), Value::NativeFunc(multiply));
        env.define(Symbol::from("/"), Value::NativeFunc(divide));
        env.define(Symbol::from("%"), Value::NativeFunc(modulo));
        env.define(Symbol::from(":"), Value::NativeFunc(from_hashmap));
        env.define(Symbol::from("format"), Value::NativeFunc(format));
        env.define(Symbol::from("bar"), Value::NativeFunc(bar));
        env.define(Symbol::from("text"), Value::NativeFunc(text));
        env.define(Symbol::from("text-strict"), Value::NativeFunc(text_strict));
        env.define(Symbol::from("text-upper"), Value::NativeFunc(text_upper));
        env.define(Symbol::from("scrolling-text"), Value::NativeFunc(scrolling_text));
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

unsafe impl Send for LispHandler {}