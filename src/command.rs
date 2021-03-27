
pub mod run {
    use std::{process::{Command, Output}};
    pub fn run_command(command: &str) -> Output {
        let tokens: Vec<&str> = command.split_whitespace().collect();
        log::debug!("Running command {}", command);
        match tokens[0] {
            "cd" => {
                run_cd(command, tokens)
            }
            _ => {
                let output = Command::new("/bin/sh")
                    .arg("-c")
                    .arg(command)
                    .output()
                    .expect("Failed to execute command");
                log::debug!("Completed command {} with result {:?}", command, output);
                output
            }
        }
    }
    
    fn run_cd(command: &str, params: Vec<&str>) -> Output {
        let output = Command::new(command)
            .args(params)
            .output()
            .expect("Failed to executed command");
        log::debug!("Completed command {} with result {:?}", command, output);
        output
    }
}

