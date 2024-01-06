use log::error;
use mlua::UserData;
use serde::{Deserialize, Serialize};
use std::process::{Child, Command, Stdio};

pub struct Process {
    process: Child,
}

impl Process {
    pub fn new(config: &Config) -> std::io::Result<Process> {
        let mut command = Command::new(&config.path);
        let mut command = command
            .args(&config.args)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        Self::extra_configuration(&mut command);

        Ok(Self {
            process: command.spawn()?,
        })
    }

    #[cfg(target_os = "windows")]
    fn extra_configuration(command: &mut Command) {
        use std::os::windows::process::CommandExt;

        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    #[cfg(target_os = "linux")]
    fn extra_configuration(_command: &mut Command) {
        // No need to explicitly disable console window
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
