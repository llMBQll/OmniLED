use std::{cell::RefCell, rc::Rc, time};
use std::collections::HashMap;
use strfmt::strfmt;

use rust_lisp::model::{FloatType, IntType, List};
use rust_lisp::model::Value;
use rust_lisp::prelude::*;
use rust_lisp::utils::{require_parameter, require_string_parameter};

use crate::lisp_handler::lisp_handler::LispHandler;

mod lisp_handler;


#[allow(unused_macros)]
macro_rules! int {
    ($a:expr) => {
        {
            Value::Int($a as i32)
        }
    }
}

#[allow(unused_macros)]
macro_rules! float {
    ($a:expr) => {
        {
            Value::Float($a as f32)
        }
    }
}

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

fn to_string(value: &Value) -> String {
    match value {
        Value::String(string) => string.to_string(),
        _ => value.to_string()
    }
}

fn load_custom_functions(env: Rc<RefCell<Env>>) {
    let mut env = env.as_ref().borrow_mut();
    env.define(
        Symbol::from("cat"),
        Value::NativeFunc(
            |_env, args| {
                let mut string = String::new();
                for arg in args {
                    match arg {
                        Value::True => { string.push_str("true"); }
                        Value::False => { string.push_str("false"); }
                        Value::Int(value) => { string.push_str(value.to_string().as_str()); }
                        Value::Float(value) => { string.push_str(value.to_string().as_str()); }
                        Value::String(value) => { string.push_str(value.as_str()); }
                        _ => { return Err(RuntimeError { msg: String::from("cat only supports boolean, numerical and string values") }); }
                    }
                }
                Ok(Value::String(string))
            }
        ),
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
                        })
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
        )
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
                    vars.insert(n.to_string(), to_string(arg));
                    n += 1;
                }

                Ok(string!(strfmt(&format, &vars).unwrap()))
            }
        )
    );
}

fn json_to_lisp(json: &serde_json::Value) -> Value {
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
            todo!("{}", "Error")
        }
        serde_json::Value::String(string) => string!(string),
        serde_json::Value::Array(array) => {
            let mut lisp_array: Vec<Value> = Vec::new();
            for value in array {
                lisp_array.push(json_to_lisp(value));
            }
            Value::List(List::from_iter(lisp_array.into_iter()))
        }
        serde_json::Value::Object(map) => {
            let mut lisp_map: HashMap<Value, Value> = HashMap::new();
            for (key, value) in map {
                lisp_map.insert(string!(key), json_to_lisp(value));
            }
            Value::HashMap(Rc::new(RefCell::new(lisp_map)))
        }
    }
}

fn pre_call(env: Rc<RefCell<Env>>, values: &HashMap<String, serde_json::Value>) {
    let mut env = env.as_ref().borrow_mut();
    for (key, value) in values {
        env.define(Symbol::from(key.as_str()), json_to_lisp(value));
    }
}

fn call(env: Rc<RefCell<Env>>, function: &Value) -> Result<Value, RuntimeError> {
    eval(env.clone(), function)
}

fn main() {
    let _handler = LispHandler::new();

    let env = Rc::new(RefCell::new(default_env()));
    load_custom_functions(env.clone());

    let str = "{\"Seconds\":10,\"Minutes\":10,\"Hours\":21,\"MonthDay\":12,\"Month\":6,\"Year\":2022,\"WeekDay\":7}";
    let json: HashMap<String, serde_json::Value> = serde_json::from_str(str).unwrap();

    let text1 = lisp! {
        (text (format "{0}:{1}:{2}" Hours Minutes Seconds))
    };
    let text2 = lisp! {
        (text (format "{0} {1}.{2}.{3}" (nth (- WeekDay 1) (list "Monday" "Tuesday" "Wednesday" "Thursday" "Friday" "Saturday" "Sunday")) MonthDay Month Year))
    };

    let begin = time::Instant::now();
    pre_call(env.clone(), &json);
    let res1 = call(env.clone(), &text1);
    let res2 = call(env.clone(), &text2);
    let end = time::Instant::now();
    println!("{} - {}", res1.unwrap(), (end - begin).as_micros());
    println!("{} - {}", res2.unwrap(), (end - begin).as_micros());
}
