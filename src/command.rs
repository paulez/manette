
pub mod run {
    use std::{process::Command, str::from_utf8};
    pub fn run_command(command: &str) -> String {
        let output = Command::new(command)
            .arg("")
            .output()
            .expect("Failed to execute command");
        String::from_utf8(output.stdout).unwrap()
    }
}
