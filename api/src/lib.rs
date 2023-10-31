use serde::{Deserialize, Serialize};
use ureq::{Agent, Error, Response};

#[derive(Debug)]
pub struct Api {
    agent: Agent,
    address: String,
    name: String,
}

impl Api {
    pub fn new(address: String, application_name: String) -> Self {
        Self {
            agent: Agent::new(),
            address: address.clone(),
            name: application_name,
        }
    }

    pub fn update<T: Serialize>(&self, data: &T) {
        self.update_with_name(data, &self.name)
    }

    pub fn update_with_name<T: Serialize>(&self, data: &T, name: &String) {
        let update_data = UpdateData { name, fields: data };

        match self.call("/update", &update_data) {
            Ok(_) => {}
            Err(err) => match err {
                Error::Status(status, response) => {
                    let response = response.into_json().unwrap_or(Reply {
                        error: Some(String::from("unknown error")),
                    });
                    println!("[{status}] {}", serde_json::to_string(&response).unwrap());
                }
                Error::Transport(transport) => println!("{transport}"),
            },
        }
    }

    fn call<T: Serialize>(&self, endpoint: &str, json: &T) -> Result<Response, Error> {
        let json = serde_json::to_string(json).unwrap();
        let url = format!("http://{}{}", self.address, endpoint);
        self.agent
            .post(url.as_str())
            .set("Content-Type", "application/json")
            .send_string(json.as_str())
    }
}

#[derive(Serialize, Deserialize)]
struct Reply {
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize)]
struct UpdateData<'a, 'b, T: Serialize> {
    name: &'a String,
    fields: &'b T,
}
