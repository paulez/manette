pub mod run {
    use cursive::Cursive;

    use crate::ui::update;

    use std::env;
    use std::fs;
    use std::path::Path;
    use std::process::{Command, Output};

    pub fn run_command(
        command: &str, s: &mut Cursive,
    ) {
        let tokens: Vec<&str> = command.split_whitespace().collect();
        log::debug!("Running command {}", command);
        match tokens[0] {
            "cd" => run_cd(tokens[0], tokens[1..].to_vec(), s),
            _ => {
                let output = Command::new("/bin/sh").arg("-c").arg(command).output();
                match output {
                    Ok(output) => {
                        log::debug!("Completed command {} with result {:?}", command, output);
                        let result = CommandResult::from_output(output);
                        update::command_output(s, result);
                    }
                    Err(output) => {
                        log::error!("Error running {} with result {:?}", command, output);
                    }
                }
            }
        }
    }

    fn run_cd(
        command: &str,
        params: Vec<&str>,
        s: &mut Cursive,
    ) {
        log::debug!("Running cd: {} to {:?}", command, &params);
        match params.first() {
            Some(path) => {
                let new_path = Path::new(path);
                log::debug!("Changing dir to {:?}", new_path);
                match env::set_current_dir(new_path) {
                    Ok(_) => {
                        log::info!("Changed dir to {:?}", new_path);
                        run_ls(s);
                    }
                    Err(error) => {
                        log::error!("Failed to change dir");
                    }
                }
            }
            None => {
                log::error!("Please provide a path to change to");
                update::show_error(s, String::from("Please provide a path to change to"));
            }
        }
    }

    fn run_ls(s: &mut Cursive) -> Result<CommandResult, std::io::Error> {
        let mut paths: Vec<String> = fs::read_dir("./")?
            .map(|res| res.unwrap().path().into_os_string().into_string().unwrap())
            .map(|path| path.strip_prefix("./").unwrap().to_string())
            .collect();
        paths.sort();
        update::file_list_view(s, paths);
        Ok(CommandResult {
            //output: paths.join("\n"),
            output: String::from(""),
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
