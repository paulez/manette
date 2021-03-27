
pub mod run {
    use std::{process::{Command, Output}};
    pub fn run_command(command: &str) -> Output {
        log::debug!("Running command {}", command);
        let output = Command::new("/bin/sh")
            .arg("-c")
            .arg(command)
            .output()
            .expect("Failed to execute command");
        log::debug!("Completed command {} with result {:?}", command, output);
        output
    }
}
