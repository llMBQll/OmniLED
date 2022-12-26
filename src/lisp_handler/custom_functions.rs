use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use rust_lisp::model::{Env, FloatType, IntType, List, RuntimeError, Symbol, Value};
use rust_lisp::utils::{require_arg, require_typed_arg};
use strfmt::strfmt;

macro_rules! int {
    ($a:expr) => {
        {
            Value::Int(($a) as IntType)
        }
    }
}

macro_rules! float {
    ($a:expr) => {
        {
            Value::Float(($a) as FloatType)
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

#[macro_export]
macro_rules! cast {
    ($target: expr, $pat: path) => {
        {
            if let $pat(a) = $target {
                a
            } else {
                panic!("mismatch variant when cast to {}", stringify!($pat));
            }
        }
    };
}

macro_rules! try_impl {
    ($a: ident, $b: ident, $func: ident, $type_a: ty, $type_b: ty, $type_to: ty, $conv: tt) => {
        {
            if let (Ok($a), Ok($b)) = (
                 TryInto::<$type_a>::try_into($a),
                 TryInto::<$type_b>::try_into($b),
            ) {
                 let x = $func($a as $type_to, $b as $type_to)?;
                 return Ok($conv!(x));
            }
        }
    }
}

macro_rules! try_perform {
    ($a: ident, $b: ident, $func_i: ident, $func_f: ident) => {
        {
            try_impl!($a, $b, $func_i, IntType, IntType, IntType, int);
            try_impl!($a, $b, $func_f, IntType, FloatType, FloatType, float);
            try_impl!($a, $b, $func_f, FloatType, IntType, FloatType, float);
            try_impl!($a, $b, $func_f, FloatType, FloatType, FloatType, float);
        }
    }
}

pub const CURRENT_INDEX_KEY: &str = "__current_index";
pub const HASH_MAP_KEY: &str = "__hash_map";
pub const RESET_FLAG_KEY: &str = "__clear_flag";

pub fn modulo(_env: Rc<RefCell<Env>>, args: &[Value]) -> Result<Value, RuntimeError> {
    let a = require_typed_arg::<IntType>("%", args, 0)?;
    let b = require_typed_arg::<IntType>("%", args, 1)?;
    match b {
        0 => Err(RuntimeError { msg: String::from("In function \"%\": attempt to calculate the remainder with a divisor of zero") }),
        _ => Ok(int!(a % b))
    }
}

pub fn add(_env: Rc<RefCell<Env>>, args: &[Value]) -> Result<Value, RuntimeError> {
    let add_int = |a, b| -> Result<IntType, RuntimeError> {
        Ok(a + b)
    };

    let add_float = |a, b| -> Result<FloatType, RuntimeError> {
        Ok(a + b)
    };

    let a = require_arg("+", args, 0)?;
    let b = require_arg("+", args, 1)?;

    try_perform!(a, b, add_int, add_float);

    Err(RuntimeError {
        msg: String::from("Function \"+\" requires arguments to be numbers"),
    })
}

pub fn subtract(_env: Rc<RefCell<Env>>, args: &[Value]) -> Result<Value, RuntimeError> {
    let subtract_int = |a, b| -> Result<IntType, RuntimeError> {
        Ok(a - b)
    };

    let subtract_float = |a, b| -> Result<FloatType, RuntimeError> {
        Ok(a - b)
    };

    let a = require_arg("-", args, 0)?;
    let b = require_arg("-", args, 1)?;

    try_perform!(a, b, subtract_int, subtract_float);

    Err(RuntimeError {
        msg: String::from("Function \"-\" requires arguments to be numbers"),
    })
}

pub fn multiply(_env: Rc<RefCell<Env>>, args: &[Value]) -> Result<Value, RuntimeError> {
    let multiply_int = |a, b| -> Result<IntType, RuntimeError> {
        Ok(a * b)
    };

    let multiply_float = |a, b| -> Result<FloatType, RuntimeError> {
        Ok(a * b)
    };

    let a = require_arg("*", args, 0)?;
    let b = require_arg("*", args, 1)?;

    try_perform!(a, b, multiply_int, multiply_float);

    Err(RuntimeError {
        msg: String::from("Function \"*\" requires arguments to be numbers"),
    })
}

pub fn divide(_env: Rc<RefCell<Env>>, args: &[Value]) -> Result<Value, RuntimeError> {
    let handle_int = |a, b| {
        if b == 0 {
            return Err(RuntimeError { msg: String::from("In function \"/\": attempt to divide zero") });
        }
        Ok(a / b)
    };

    let handle_float = |a, b| {
        if b == 0.0 {
            return Err(RuntimeError { msg: String::from("In function \"/\": attempt to divide zero") });
        }
        return Ok(a / b);
    };

    let a = require_arg("/", args, 0)?;
    let b = require_arg("/", args, 1)?;

    try_perform!(a, b, handle_int, handle_float);

    Err(RuntimeError {
        msg: String::from("Function \"/\" requires arguments to be numbers"),
    })
}

pub fn format(_env: Rc<RefCell<Env>>, args: &[Value]) -> Result<Value, RuntimeError> {
    let format = require_typed_arg::<&String>("format", args, 0)?;

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

pub fn from_hashmap(env: Rc<RefCell<Env>>, args: &[Value]) -> Result<Value, RuntimeError> {
    let string = require_typed_arg::<&String>(":", args, 0)?;

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

pub fn bar(_env: Rc<RefCell<Env>>, args: &[Value]) -> Result<Value, RuntimeError> {
    let bar = require_arg("bar", args, 0)?;

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

pub fn text(_env: Rc<RefCell<Env>>, args: &[Value]) -> Result<Value, RuntimeError> {
    let text = require_arg("text", args, 0)?;

    Ok(list![string!("text"), string!(value_to_string(text))])
}

pub fn text_strict(_env: Rc<RefCell<Env>>, args: &[Value]) -> Result<Value, RuntimeError> {
    let text = require_arg("text-strict", args, 0)?;

    Ok(list![string!("text-strict"), string!(value_to_string(text))])
}

pub fn text_upper(_env: Rc<RefCell<Env>>, args: &[Value]) -> Result<Value, RuntimeError> {
    let text = require_arg("text-upper", args, 0)?;

    Ok(list![string!("text-upper"), string!(value_to_string(text))])
}

pub fn scrolling_text(env: Rc<RefCell<Env>>, args: &[Value]) -> Result<Value, RuntimeError> {
    let mut env = env.as_ref().borrow_mut();

    let text = require_arg("scrolling-text", args, 0)?;
    let text = string!(value_to_string(text));

    // Get HashMap associated with currently processed local env
    let map = env.get(&Symbol::from(HASH_MAP_KEY)).unwrap();
    let map = cast!(map, Value::HashMap);
    let mut map = map.as_ref().borrow_mut();

    // Get index of currently processed handler -- value is incremented inside 'process_handlers' function
    let key = env.get(&Symbol::from(CURRENT_INDEX_KEY)).unwrap();

    // Get already existing text-count pair or create a new one
    // Set value to '-1' as it will be incremented immediately
    let entry = map.entry(key).or_insert(list![text.clone(), int!(-1)]);
    let list = cast!(entry, Value::List);

    let mut iter = list.into_iter();
    let previous_text = iter.next().unwrap();
    let count = cast!(iter.next().unwrap(), Value::Int);

    // Count depending on value
    //  - new value / value changed -> count = 0
    //  - value didn't change       -> count = previous_count + 1
    let count = if text == previous_text {
        count + 1
    } else {
        env.set(Symbol::from(RESET_FLAG_KEY), Value::True).unwrap();
        0
    };

    *entry = list![text.clone(), int!(count)];

    Ok(list![string!("scrolling-text"), text, int!(count)])
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(string) => string.to_string(),
        _ => value.to_string()
    }
}