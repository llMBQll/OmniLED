use std::io::{Write};


use std::process::{Child, Command, ExitStatus, Stdio};


pub struct Plugin {
    name: String,
    process: Child,
}

impl Plugin {
    pub fn new(name: String, addr: &String) -> std::io::Result<Plugin> {
        let process = Command::new("runner")
            .arg(&name)
            .arg(addr)
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .spawn()?;

        Ok(Self {
            name,
            process,
        })
    }

    pub fn stop(&mut self) -> std::io::Result<ExitStatus> {
        self.process.stdin.as_ref().unwrap().write(b"stop\n")?;
        self.process.wait()
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}