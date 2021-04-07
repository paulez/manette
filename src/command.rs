
pub mod run {
    use std::{process::{Command, Output}};
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};

    use crate::RunState;
    pub fn run_command(command: &str, runstate: &RunState) -> Result<CommandResult, std::io::Error> {
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
                        Ok(CommandResult::from_output(output))
                    }
                    Err(output) => {
                        log::error!("Error running {} with result {:?}", command, output);
                        Err(output)
                    }
                }
            }
        }
    }

    fn run_cd(command: &str, mut params: Vec<&str>, runstate: &RunState) -> Result<CommandResult, std::io::Error>{
        log::debug!("Running cd: {} to {:?}", command, &params);
        match params.first() {
            Some(path) => {
                let new_path = Path::new(path);
                log::debug!("Changing dir to {:?}", new_path);
                match env::set_current_dir(new_path) {
                    Ok(result) => {
                        log::info!("Changed dir to {:?}", new_path);
                        Ok(CommandResult{
                            output: String::from(format!("Changed dir to {:?}", new_path)),
                            error_output: String::from(""),
                        })
                    },
                    Err(error) => {
                        log::error!("Failed to change dir");
                        Err(error)
                    }
                }
            },
            None => {
                log::error!("Please provide a path to change to");
                Ok(CommandResult{
                    output: String::from(""),
                    error_output: String::from("Please provide a path to change to"),
                })
            }
        }
    }

    fn run_ls(command: &str, mut_params: Vec<&str>, runstate: &RunState) -> Result<CommandResult, std::io::Error>{
        let paths: Vec<_> = fs::read_dir("./")?
            .map(|res| res.unwrap().path().into_os_string().into_string().unwrap())
            .collect();
        Ok(CommandResult{
            output: String::from(paths.join("\n")),
            error_output: String::from(""),
        })
    }

    pub struct CommandResult {
        pub output: String,
        pub error_output: String,
    }

    impl  CommandResult {

        fn from_output(output: Output) -> CommandResult {
            CommandResult{
                output: String::from_utf8(output.stdout).unwrap(),
                error_output: String::from_utf8(output.stderr).unwrap(),
            }
        }
    }
}


