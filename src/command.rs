pub mod run {
    use cursive::Cursive;

    use crate::ui::update;

    use std::env;
    use std::fs;
    use std::path::Path;
    use std::process::{Command, Output};

    pub fn run_command(command: &str, s: &mut Cursive) {
        let tokens: Vec<&str> = command.split_whitespace().collect();
        log::debug!("Running command {}", command);
        match tokens.is_empty() {
            true => update::show_error(s, "Please enter a command.".to_string()),
            false => match tokens[0] {
                "cd" => run_cd(tokens[1..].to_vec(), s),
                "ls" => run_ls(tokens[1..].to_vec(), s),
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
            },
        }
    }

    pub fn submit_file(s: &mut Cursive, filename: &str) {
        let metadata = fs::metadata(filename);

        match metadata {
            Ok(metadata) => {
                log::debug!("File metadata: {:?}", metadata);
                if metadata.is_dir() {
                    log::debug!("{} is a directory", filename);
                    run_cd([filename].to_vec(), s);
                } else if metadata.is_file() {
                    log::debug!("{} is a file", filename);
                } else {
                    log::debug!("{} is something else", filename);
                }
            }
            Err(error) => {
                log::error!("Failed to read metadata {:?}", error);
                update::show_error(s, format!("Failed to read metadata {:?}", error))
            }
        }
    }

    fn run_cd(params: Vec<&str>, s: &mut Cursive) {
        log::debug!("Running cd to {:?}", &params);
        match params.first() {
            Some(path) => {
                let new_path = Path::new(path);
                log::debug!("Changing dir to {:?}", new_path);
                match env::set_current_dir(new_path) {
                    Ok(_) => {
                        log::info!("Changed dir to {:?}", new_path);
                        run_ls(["./"].to_vec(), s);
                    }
                    Err(error) => {
                        log::error!("Failed to change dir {:?}", error);
                        update::show_error(s, format!("Failed to change dir {:?}", error))
                    }
                }
            }
            None => {
                log::error!("Please provide a path to change to");
                update::show_error(s, "Please provide a path to change to".to_string());
            }
        }
    }

    fn run_ls(params: Vec<&str>, s: &mut Cursive) {
        let dir = match params.len() {
            0 => "./",
            1 => match params.first() {
                Some(param) => param,
                None => {
                    update::show_error(s, "Please provide one argument".to_string());
                    return;
                }
            },
            _ => {
                update::show_error(s, "Please provide one argument".to_string());
                return;
            }
        };
        match fs::read_dir(dir) {
            Ok(paths) => {
                let mut path_strings: Vec<String> = Vec::new();

                for path in paths {
                    match path {
                        Ok(path) => {
                            let path = path.path();
                            let path = match path.strip_prefix(dir) {
                                Ok(path_without_prefix) => path_without_prefix.to_path_buf(),
                                Err(error) => {
                                    log::error!(
                                        "Cannot remove prefix from {:?}: {:?}",
                                        path,
                                        error
                                    );
                                    path
                                }
                            };
                            let path = path.into_os_string().into_string();
                            match path {
                                Ok(path) => path_strings.push(path.to_string()),
                                Err(error) => {
                                    update::show_error(
                                        s,
                                        format!("Error converting path to string: {:?}", error),
                                    );
                                }
                            }
                        }
                        Err(error) => {
                            update::show_error(s, format!("Error listing path: {:?}", error));
                        }
                    }
                }

                path_strings.sort();
                match env::current_dir() {
                    Ok(current_dir) => {
                        if current_dir != Path::new("/").to_path_buf() {
                            path_strings.insert(0, "..".to_string());
                        }
                    }
                    Err(error) => log::error!("Cannot get current directory: {:?}", error),
                }
                update::file_list_view(s, path_strings);
            }
            Err(error) => {
                update::show_error(s, error.to_string());
            }
        }
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
