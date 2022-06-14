use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use rust_lisp::model::{FloatType, IntType};
use rust_lisp::prelude::*;
use rust_lisp::utils::{require_int_parameter, require_parameter, require_string_parameter};
use strfmt::strfmt;

macro_rules! int {
    ($a:expr) => {
        {
            Value::Int($a as IntType)
        }
    }
}

macro_rules! float {
    ($a:expr) => {
        {
            Value::Float($a as FloatType)
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

pub fn modulo(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
    let a = require_int_parameter("%", args, 0)?;
    let b = require_int_parameter("%", args, 1)?;
    match b {
        0 => Err(RuntimeError { msg: String::from("In function \"%\": attempt to calculate the remainder with a divisor of zero") }),
        _ => Ok(int!(a % b))
    }
}

pub fn divide(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
    let handle_int = |a, b| {
        match b {
            0 => Err(RuntimeError { msg: String::from("In function \"%\": attempt to calculate the remainder with a divisor of zero") }),
            _ => Ok(a / b)
        }
    };

    #[allow(illegal_floating_point_literal_pattern)]
        let handle_float = |a, b| {
        match b {
            0.0f32 => Err(RuntimeError { msg: String::from("In function \"%\": attempt to calculate the remainder with a divisor of zero") }),
            _ => Ok(a / b)
        }
    };

    let a = require_parameter("/", args, 0)?;
    let b = require_parameter("/", args, 1)?;

    if let (Ok(a), Ok(b)) = (
        TryInto::<IntType>::try_into(a),
        TryInto::<IntType>::try_into(b),
    ) {
        let x = handle_int(a, b)?;
        return Ok(int!(x));
    }

    if let (Ok(a), Ok(b)) = (
        TryInto::<FloatType>::try_into(a),
        TryInto::<IntType>::try_into(b),
    ) {
        let x = handle_float(a, b as FloatType)?;
        return Ok(float!(x));
    }

    if let (Ok(a), Ok(b)) = (
        TryInto::<IntType>::try_into(a),
        TryInto::<FloatType>::try_into(b),
    ) {
        let x = handle_float(a as FloatType, b)?;
        return Ok(float!(x));
    }

    if let (Ok(a), Ok(b)) = (
        TryInto::<FloatType>::try_into(a),
        TryInto::<FloatType>::try_into(b),
    ) {
        let x = handle_float(a, b)?;
        return Ok(float!(x));
    }

    Err(RuntimeError {
        msg: String::from("Function \"/\" requires arguments to be numbers"),
    })
}

pub fn format(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
    let format = require_string_parameter("format", args, 0)?;

    let mut vars = HashMap::new();
    let mut n = 0;
    let mut iter = args.into_iter();
    iter.next();
    for arg in iter {
        vars.insert(n.to_string(), value_to_string(arg));
        n += 1;
    }

    Ok(string!(strfmt(&format, &vars).unwrap()))
}

pub fn from_hashmap(env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
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
            _ => { return Err(RuntimeError { msg: String::from("Expected List or HashMap") }); }
        }
    }

    Ok(val)
}

pub fn bar(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
    let bar = require_parameter("bar", args, 0)?;

    if let Ok(bar) = TryInto::<FloatType>::try_into(bar) {
        return Ok(list![string!("bar"), float!(bar)]);
    }
    if let Ok(bar) = TryInto::<IntType>::try_into(bar) {
        return Ok(list![string!("bar"), float!(bar)]);
    }

    Err(RuntimeError {
        msg: String::from("Function \"bar\" requires argument to be a number"),
    })
}

pub fn text(_env: Rc<RefCell<Env>>, args: &Vec<Value>) -> Result<Value, RuntimeError> {
    let text = require_parameter("text", args, 0)?;

    Ok(list![string!("text"), string!(value_to_string(text))])
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(string) => string.to_string(),
        _ => value.to_string()
    }
}