use std::collections::HashMap;
use std::time;

use rust_lisp::model::Value;
use rust_lisp::parser::parse;

use crate::lisp_handler::lisp_handler::LispHandler;

mod lisp_handler;


fn main() {
    let mut handlers = Vec::<Value>::new();
    let mut ast = parse(r#" (text (format "{0:0>2}:{1:0>2}" CLOCK:Hours CLOCK:Minutes)) "#);
    handlers.push(ast.next().unwrap().unwrap());
    let mut ast = parse(r#" (bar (/ (* CLOCK:Seconds 100) 59)) "#);
    handlers.push(ast.next().unwrap().unwrap());
    let mut ast = parse(r#" (text (format "{0} {1}.{2:0>2}.{3:0>2}" (nth CLOCK:WeekDay (list "Mon" "Tue" "Wed" "Thu" "Fri" "Sat" "Sun")) CLOCK:MonthDay CLOCK:Month (% CLOCK:Year 100))) "#);
    handlers.push(ast.next().unwrap().unwrap());
    let mut ast = parse(r#" (text CLOCK:Seconds) "#);
    handlers.push(ast.next().unwrap().unwrap());

    let str = "{\"Seconds\":59,\"Minutes\":6,\"Hours\":0,\"MonthDay\":12,\"Month\":6,\"Year\":2022,\"WeekDay\":0}";
    let json: HashMap<String, serde_json::Value> = serde_json::from_str(str).unwrap();

    let name = String::from("CLOCK");

    let mut handler = LispHandler::new();
    handler.register(&name);

    let begin = time::Instant::now();

    handler.update(&name, &json);
    let results = handler.process_handlers(&handlers);

    let end = time::Instant::now();
    println!("{}", (end - begin).as_micros());

    for result in results {
        println!("{}", result);
    }
}
