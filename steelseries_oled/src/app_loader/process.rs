use log::error;
use mlua::{ErrorContext, FromLua, UserData};
use oled_derive::FromLuaTable;
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
        // No extra configuration required
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

#[derive(Debug, Clone, FromLuaTable)]
pub struct Config {
    path: String,
    #[mlua(default)]
    args: Vec<String>,
}

impl UserData for Config {}
