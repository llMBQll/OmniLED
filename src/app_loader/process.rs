use log::error;
use mlua::UserData;
use serde::{Deserialize, Serialize};
use std::process::{Child, Command, Stdio};

pub struct Process {
    process: Child,
}

impl Process {
    pub fn new(config: &Config) -> std::io::Result<Process> {
        let process = Command::new(&config.path)
            .args(&config.args)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        Ok(Self { process })
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        match self.process.kill() {
            Ok(_) => {}
            Err(err) => {
                error!("{}", err.to_string())
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    path: String,
    #[serde(default)]
    args: Vec<String>,
}

impl UserData for Config {}
