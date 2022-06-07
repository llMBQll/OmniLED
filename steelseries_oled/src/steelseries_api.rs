// use std::fs::File;
// use std::io::BufReader;
// use std::{thread, time};
// use serde_json::{json, Value};
// use ureq::{Agent, Response};
//
//
// pub struct SteelSeriesAPI {
//     agent: Agent,
//     address: Option<String>
// }
//
// impl SteelSeriesAPI {
//     pub fn new() -> Self {
//         Self {
//             agent: Agent::new(),
//             address: match Self::get_address() {
//                 Ok(address) => Some(address),
//                 Err(_) => None
//             }
//         }
//     }
//
//     fn get_address() -> Result<String, Box<dyn std::error::Error>> {
//         // Missing directories are fatal errors
//         let program_data = std::env::var("PROGRAMDATA").expect("PROGRAMDATA env variable not found");
//         let dir = format!("{}\\SteelSeries\\SteelSeries Engine 3", program_data);
//
//         // TODO check if directory exists
//
//         let path = format!("{}\\coreProps.json", dir);
//         let file = File::open(path)?;
//         let reader = BufReader::new(file);
//         let json: Value = serde_json::from_reader(reader)?;
//         let address = json["address"].as_str()?;
//         Ok(String::from(address))
//     }
//
//     fn wait_for_address(&mut self) {
//         if self.address.is_some() {
//             return;
//         }
//         loop {
//             match Self::get_address() {
//                 Ok(address) => self.address = Some(address),
//                 Err(e) => {
//                     println!("Error: {}": e);
//                     thread::sleep(time::Duration::from_secs(5));
//                 }
//             }
//         }
//     }
//
//     fn call (&mut self, endpoint: &str, json: &str) -> Result<Response, ureq::Error> {
//         self.wait_for_address();
//
//         let address = &self.address.unwrap();
//         let url = format!("http://{}/{}", address, endpoint);
//         self.agent.post(url.as_str())
//             .set("Content-Type", "application/json")
//             .send_string(json)
//     }
//
//     pub fn game_metadata(&mut self, json: &str) -> Result<Response, ureq::Error> {
//         self.call(json, "game_metadata")
//     }
//
//     pub fn bind_game_event(&mut self, json: &str) -> Result<Response, ureq::Error> {
//         self.call(json, "bind_game_event")
//     }
//
//     pub fn game_event(&mut self, json: &str) -> Result<Response, ureq::Error> {
//         self.call(json, "game_event")
//     }
// }