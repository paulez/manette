
pub mod run {
    use std::{process::{Command, Output}};
    pub fn run_command(command: &str) -> Result<Output, std::io::Error> {
        let tokens: Vec<&str> = command.split_whitespace().collect();
        log::debug!("Running command {}", command);
        match tokens[0] {
            "cd" => {
                run_cd(tokens[0], tokens[1..].to_vec())
            }
            _ => {
                let output = Command::new("/bin/sh")
                    .arg("-c")
                    .arg(command)
                    .output();
                match output {
                    Ok(output) => {
                        log::debug!("Completed command {} with result {:?}", command, output);
                        Ok(output)
                    }
                    Err(output) => {
                        log::error!("Error running {} with result {:?}", command, output);
                        Err(output)
                    }
                }
            }
        }
    }

    fn run_cd(command: &str, params: Vec<&str>) -> Result<Output, std::io::Error>{
        log::debug!("Running cd: {} to {:?}", command, &params);
        let output = Command::new(command)
            .args(&params)
            .output();
        match output {
            Ok(output) => {
                log::debug!("Completed command {} with result {:?}", command, output);
                Ok(output)
            }
            Err(output) => {
                log::error!("Error running {} with result {:?}", command, output);
                Err(output)
            }
        }
    }
}

