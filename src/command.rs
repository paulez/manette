
pub mod run {
    use std::{process::{Command, Output}};
    use std::env;
    use std::path::Path;

    use crate::RunState;
    pub fn run_command(command: &str, runstate: &RunState) -> Result<Output, std::io::Error> {
        let tokens: Vec<&str> = command.split_whitespace().collect();
        log::debug!("Running command {}", command);
        match tokens[0] {
            "cd" => {
                run_cd(tokens[0], tokens[1..].to_vec(), &runstate)
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

    fn run_cd(command: &str, mut params: Vec<&str>, runstate: &RunState) -> Result<Output, std::io::Error>{
        log::debug!("Running cd: {} to {:?}", command, &params);
        match params.first() {
            Some(path) => {
                let new_path = Path::new(path);
                log::debug!("Changing dir to {:?}", new_path);
                match env::set_current_dir(new_path) {
                    Ok(result) => {
                        log::info!("Changed dir to {:?}", new_path);
                    },
                    Err(_) => {
                        log::error!("Failed to change dir");
                    }
                };
            },
            None => log::error!("Please provide a patch to change to"),
        };

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

