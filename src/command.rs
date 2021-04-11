pub mod run {
    use cursive::{Cursive, CursiveExt};

    use crate::ui::update;

    use std::fs;
    use std::path::Path;
    use std::process::{Command, Output};
    use std::{cmp::Ordering, env, os::unix::prelude::PermissionsExt};

    pub fn run_command(command: &str, s: &mut Cursive) {
        let tokens: Vec<&str> = command.split_whitespace().collect();
        log::debug!("Running command {}", command);
        match tokens.is_empty() {
            true => update::show_error(s, "Please enter a command.".to_string()),
            false => match tokens[0] {
                "cd" => run_cd(tokens[1..].to_vec(), s),
                "ls" => run_ls(tokens[1..].to_vec(), s),
                "vim" => run_detached_command(tokens[0], tokens[1..].to_vec(), s),
                "emacs" => run_detached_command(tokens[0], tokens[1..].to_vec(), s),
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

    fn run_detached_command(command: &str, params: Vec<&str>, s: &mut Cursive) {
        update::clear_command(s);
        s.quit();
        let exit_status = Command::new(command).args(params).status();
        s.run();
        match exit_status {
            Ok(exit_status) => {
                log::debug!(
                    "Completed command {} with status {:?}",
                    command,
                    exit_status
                );
            }
            Err(output) => {
                log::error!("Error running {} with result {:?}", command, output);
            }
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
                let mut path_strings: Vec<FileEntry> = Vec::new();

                for entry in paths {
                    match entry {
                        Ok(entry) => {
                            let path = entry.path();
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
                            let path = path.to_str();
                            match path {
                                Some(path) => {
                                    let metadata = entry.metadata();
                                    let filetype: FileType = match metadata {
                                        Ok(metadata) => {
                                            if metadata.is_dir() {
                                                FileType::Directory
                                            } else if metadata.file_type().is_symlink() {
                                                FileType::Symlink
                                            } else if metadata.permissions().mode() & 0o111 != 0 {
                                                FileType::Executable
                                            } else if metadata.is_file() {
                                                FileType::File
                                            } else {
                                                FileType::Unknown
                                            }
                                        }
                                        Err(error) => {
                                            log::error!("Cannot get metadata: {:?}", error);
                                            FileType::Unknown
                                        }
                                    };
                                    path_strings.push(FileEntry {
                                        filename: path.to_string(),
                                        filetype,
                                    });
                                }
                                None => {
                                    update::show_error(
                                        s,
                                        format!("Error converting path to string: {:?}", entry),
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
                            path_strings.insert(
                                0,
                                FileEntry {
                                    filename: "..".to_string(),
                                    filetype: FileType::Directory,
                                },
                            );
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

    pub enum FileType {
        Directory,
        Executable,
        File,
        Symlink,
        Unknown,
    }

    pub struct FileEntry {
        pub filename: String,
        pub filetype: FileType,
    }

    impl Ord for FileEntry {
        fn cmp(&self, other: &Self) -> Ordering {
            self.filename.cmp(&other.filename)
        }
    }

    impl PartialOrd for FileEntry {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl PartialEq for FileEntry {
        fn eq(&self, other: &Self) -> bool {
            self.filename == other.filename
        }
    }

    impl Eq for FileEntry {}
}
