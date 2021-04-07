pub mod run {
    use std::env;
    use std::fs;
    use std::path::Path;
    use std::process::{Command, Output};

    pub fn run_command(
        command: &str,
    ) -> Result<CommandResult, std::io::Error> {
        let tokens: Vec<&str> = command.split_whitespace().collect();
        log::debug!("Running command {}", command);
        match tokens[0] {
            "cd" => run_cd(tokens[0], tokens[1..].to_vec()),
            _ => {
                let output = Command::new("/bin/sh").arg("-c").arg(command).output();
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

    fn run_cd(
        command: &str,
        params: Vec<&str>,
    ) -> Result<CommandResult, std::io::Error> {
        log::debug!("Running cd: {} to {:?}", command, &params);
        match params.first() {
            Some(path) => {
                let new_path = Path::new(path);
                log::debug!("Changing dir to {:?}", new_path);
                match env::set_current_dir(new_path) {
                    Ok(_) => {
                        log::info!("Changed dir to {:?}", new_path);
                        run_ls()
                    }
                    Err(error) => {
                        log::error!("Failed to change dir");
                        Err(error)
                    }
                }
            }
            None => {
                log::error!("Please provide a path to change to");
                Ok(CommandResult {
                    output: String::from(""),
                    error_output: String::from("Please provide a path to change to"),
                })
            }
        }
    }

    fn run_ls() -> Result<CommandResult, std::io::Error> {
        let mut paths: Vec<String> = fs::read_dir("./")?
            .map(|res| res.unwrap().path().into_os_string().into_string().unwrap())
            .map(|path| path.strip_prefix("./").unwrap().to_string())
            .collect();
        paths.sort();
        Ok(CommandResult {
            output: paths.join("\n"),
            error_output: String::from(""),
        })
    }

    pub struct CommandResult {
        pub output: String,
        pub error_output: String,
    }

    impl CommandResult {
        fn from_output(output: Output) -> CommandResult {
            CommandResult {
                output: String::from_utf8(output.stdout).unwrap(),
                error_output: String::from_utf8(output.stderr).unwrap(),
            }
        }
    }
}
