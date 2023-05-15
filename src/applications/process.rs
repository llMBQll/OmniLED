use serde::{Deserialize, Serialize};
use std::process::{Child, Command, Stdio};
use mlua::UserData;

pub struct Process {
    process: Child,
}

impl Process {
    pub fn new(config: &Config) -> std::io::Result<Process> {
        let process = Command::new(&config.path)
            .args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        Ok(Self {
            process,
        })
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        self.process.wait().expect("Command wasn't running");
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    path: String,
    args: Vec<String>,
}

impl UserData for Config {}