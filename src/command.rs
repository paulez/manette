/*

Copyright or © or Copr. Paul Ezvan (2021-2022)

paul@ezvan.fr

This software is a computer program whose purpose is to provide a terminal file explorer.

This software is governed by the CeCILL license under French law and
abiding by the rules of distribution of free software.  You can  use,
modify and/ or redistribute the software under the terms of the CeCILL
license as circulated by CEA, CNRS and INRIA at the following URL
"http://www.cecill.info".

As a counterpart to the access to the source code and  rights to copy,
modify and redistribute granted by the license, users are provided only
with a limited warranty  and the software's author,  the holder of the
economic rights,  and the successive licensors  have only  limited
liability.

In this respect, the user's attention is drawn to the risks associated
with loading,  using,  modifying and/or developing or reproducing the
software by the user in light of its specific status of free software,
that may mean  that it is complicated to manipulate,  and  that  also
therefore means  that it is reserved for developers  and  experienced
professionals having in-depth computer knowledge. Users are therefore
encouraged to load and test the software's suitability as regards their
requirements in conditions enabling the security of their systems and/or
data to be ensured and,  more generally, to use and operate it in the
same conditions as regards security.

The fact that you are presently reading this means that you have had
knowledge of the CeCILL license and that you accept its terms.

*/

pub mod run {
    use cursive::{Cursive, CursiveExt};

    use crate::file::filetype;
    use crate::file::filetype::FileType;
    use crate::file::open;
    use crate::ui::update;
    use crate::userenv::userenv;

    use std::fs;
    use std::path::Path;
    use std::process::{Command, Output};
    use std::{cmp::Ordering, env};

    pub fn run_command(command: &str, s: &mut Cursive) {
        let tokens: Vec<&str> = command.split_whitespace().collect();
        log::debug!("Running command {}", command);
        match tokens.is_empty() {
            true => update::show_error(s, "Please enter a command.".to_string()),
            false => match tokens[0] {
                "cd" => run_cd(tokens[1..].to_vec(), s),
                "ls" => run_ls(tokens[1..].to_vec(), s),
                "emacs" | "vim" | "less" => {
                    run_detached_command(tokens[0], tokens[1..].to_vec(), s)
                }
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
                    open::open_file(s, filename);
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

    pub fn edit_file(s: &mut Cursive, filename: &str) {
        let metadata = fs::metadata(filename);

        match metadata {
            Ok(metadata) => {
                if metadata.is_file() {
                    let editor = userenv::editor();
                    let tokens: Vec<&str> = editor.split_whitespace().collect();
                    let command = match tokens.is_empty() {
                        true => "vim",
                        false => tokens[0],
                    };
                    let args = match tokens.is_empty() {
                        true => vec![filename],
                        false => {
                            let mut all_args = tokens[1..].to_vec();
                            all_args.push(filename);
                            all_args
                        }
                    };
                    run_detached_command(command, args, s);
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
                                    let filetype: FileType = filetype::get_type(entry);
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

    #[cfg(test)]
    mod tests {
        use super::*;
        use cursive;

        #[test]
        fn test_ls() {
            let mut test_cursive = cursive::dummy();
            let test_params = vec!["."];
            run_ls(test_params, &mut test_cursive);
        }
    }
}
