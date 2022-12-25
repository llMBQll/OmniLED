use std::process::{Child, Command, Stdio};


pub struct Plugin {
    process: Child,
}

impl Plugin {
    pub fn new(path: &String, addr: &String) -> std::io::Result<Plugin> {
        let process = Command::new(path)
            .arg(addr)
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        Ok(Self {
            process,
        })
    }
}

impl Drop for Plugin {
    fn drop(&mut self) {
        self.process.wait().expect("Command wasn't running");
    }
}