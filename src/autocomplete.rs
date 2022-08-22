/*

Copyright or © or Copr. Paul Ezvan (2022)

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
use std::cmp::Ordering;

#[derive(Clone, Debug, Eq)]
pub struct CompletionChoice {
    pub label: String,
    pub completion: String,
}

impl PartialEq for CompletionChoice {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}

impl PartialEq<String> for CompletionChoice {
    fn eq(&self, other: &String) -> bool {
        &self.label == other
    }
}

impl Ord for CompletionChoice {
    fn cmp(&self, other: &Self) -> Ordering {
        self.label.cmp(&other.label)
    }
}

impl PartialOrd for CompletionChoice {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub mod autocomplete {
    use crate::autocomplete::CompletionChoice;
    use crate::userenv::userenv;
    use anyhow::Result;
    use std::ffi::OsString;
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::{env, error, fmt, fs, io};

    enum CompletionType {
        Executable,
        File,
    }

    // Represents command and arguments provided on the CLI
    #[derive(Clone, Debug)]
    struct CommandArguments {
        command: String,
        arguments: Vec<String>,
    }

    impl fmt::Display for CommandArguments {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{} {}", self.command, self.arguments.join(" "))
        }
    }

    #[derive(Debug)]
    enum AutoCompleteError {
        NonUnicodePath(OsString),
    }

    impl fmt::Display for AutoCompleteError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                AutoCompleteError::NonUnicodePath(_) => write!(f, "Invalid Unicode path"),
            }
        }
    }
    impl error::Error for AutoCompleteError {
        fn description(&self) -> &str {
            match *self {
                AutoCompleteError::NonUnicodePath(_) => "Invalid Unicode path",
            }
        }
    }

    pub fn autocomplete(command: &str) -> Result<Vec<CompletionChoice>> {
        let command_args = build_command_arguments(command);
        let mut choices: Vec<CompletionChoice> = match get_completion_type(&command_args) {
            CompletionType::File => autocomplete_path(command_args, None)?,
            CompletionType::Executable => userenv::path()
                .split(':')
                .filter_map(|path| {
                    executables_in_path_with_prefix(path, &command_args.command).ok()
                })
                .flatten()
                .collect::<Vec<CompletionChoice>>(),
        };
        choices.sort();
        choices.dedup();
        Ok(choices)
    }

    fn get_completion_type(command_args: &CommandArguments) -> CompletionType {
        if !command_args.arguments.is_empty() {
            CompletionType::File
        } else {
            CompletionType::Executable
        }
    }

    fn build_command_arguments(input_command: &str) -> CommandArguments {
        let items: Vec<&str> = input_command.split(' ').collect();
        let mut arguments: Vec<String> = Vec::new();
        let mut command: String = String::new();
        for (i, item) in items.iter().enumerate() {
            if i == 0 {
                command = item.to_string();
            } else {
                arguments.push(item.to_string());
            }
        }
        CommandArguments { command, arguments }
    }

    fn executables_in_path_with_prefix(path: &str, prefix: &str) -> Result<Vec<CompletionChoice>> {
        let paths = fs::read_dir(path)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;
        let paths = paths
            .iter()
            .filter_map(|p| p.file_name().map(|filename| (p, filename)))
            .filter_map(|(p, name)| name.to_str().map(|name| (p, name)))
            .collect::<Vec<(&PathBuf, &str)>>();
        let completions = paths
            .iter()
            .filter(|(p, _name)| p.starts_with(prefix))
            .filter_map(|(p, name)| match p.metadata() {
                Ok(metadata) => Some((metadata, name)),
                Err(err) => {
                    log::error!("Cannot read metadata: {:?}", err);
                    None
                }
            })
            .filter(|(metadata, _name)| {
                let permissions = metadata.permissions();
                permissions.mode() & 0o111 != 0
            })
            .map(|(_p, name)| CompletionChoice {
                label: name.to_string(),
                completion: name.to_string(),
            })
            .collect::<Vec<CompletionChoice>>();
        Ok(completions)
    }

    fn autocomplete_path(
        command_args: CommandArguments,
        current_dir: Option<PathBuf>,
    ) -> Result<Vec<CompletionChoice>> {
        log::debug!("Autocompleting path: {:?}", command_args);
        let empty_arg = String::from("");
        let current_arg = match command_args.arguments.last() {
            Some(arg) => arg,
            None => &empty_arg,
        };
        let mut current_dir = match current_dir {
            None => env::current_dir()?,
            Some(path) => path,
        };
        let prefix = directory_from_path(current_arg);
        log::debug!("Prefix is {:?}", prefix);
        match prefix {
            None => (),
            Some(ref prefix_dir) => current_dir.push(prefix_dir.clone()),
        }
        log::debug!("Listing dir: {:?}", current_dir);
        let paths = fs::read_dir(current_dir)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;
        log::debug!("Paths read: {:?}", paths);
        let paths = paths
            .iter()
            .filter_map(|path| path_to_name(path))
            .map(|path| match prefix {
                None => path,
                Some(ref prefix_dir) => prefix_dir.clone() + &path,
            })
            .collect::<Vec<String>>();
        log::debug!("Path strings: {:?}", paths);
        let completions = paths
            .iter()
            .filter(|p| p.starts_with(current_arg))
            .map(|p| CompletionChoice {
                label: p.to_string(),
                completion: path_full_completion(command_args.clone(), p.to_string()),
            })
            .collect::<Vec<CompletionChoice>>();
        Ok(completions)
    }

    fn path_full_completion(mut args: CommandArguments, completion: String) -> String {
        if !args.arguments.is_empty() {
            args.arguments.pop();
        }
        args.arguments.push(completion);
        log::debug!("Pushing completion: {:?}", args.to_string());
        args.to_string()
    }

    fn directory_from_path(path: &str) -> Option<String> {
        path.rfind('/')
            .map(|last_slash| path.split_at(last_slash + 1).0.to_string())
    }

    fn path_to_name(path: &Path) -> Option<String> {
        match path.file_name() {
            Some(name) => match name.to_str() {
                Some(name) => {
                    if path.is_dir() {
                        Some(format!("{}/", name))
                    } else {
                        Some(name.to_string())
                    }
                }
                None => None,
            },
            None => None,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn test_autocomplete_path() {
            //TODO: use random dir name
            fs::create_dir_all("/tmp/manette/test").unwrap();
            fs::write("/tmp/manette/test/a", "").unwrap();
            fs::write("/tmp/manette/test/b", "").unwrap();
            let test_path = PathBuf::from("/tmp/manette/test");
            let test_args = CommandArguments {
                command: String::from("ls"),
                arguments: vec![],
            };
            let mut results =
                autocomplete_path(test_args.clone(), Some(test_path.clone())).unwrap();
            results.sort();
            let mut expected_results = vec![
                CompletionChoice {
                    label: String::from("a"),
                    completion: String::from("ls a"),
                },
                CompletionChoice {
                    label: String::from("b"),
                    completion: String::from("ls b"),
                },
            ];
            assert_eq!(results, expected_results,);

            // add a directory in completion choices
            fs::create_dir("/tmp/manette/test/dir").unwrap();
            let mut results =
                autocomplete_path(test_args.clone(), Some(test_path.clone())).unwrap();
            results.sort();

            expected_results.push(CompletionChoice {
                label: String::from("dir/"),
                completion: String::from("ls dir/"),
            });
            assert_eq!(results, expected_results,);

            // list files in subdir
            fs::write("/tmp/manette/test/dir/a", "").unwrap();
            fs::write("/tmp/manette/test/dir/b", "").unwrap();
            let test_args = CommandArguments {
                command: String::from("ls"),
                arguments: vec![String::from("dir/")],
            };
            let mut results =
                autocomplete_path(test_args.clone(), Some(test_path.clone())).unwrap();
            results.sort();
            let expected_results = vec![
                CompletionChoice {
                    label: String::from("dir/a"),
                    completion: String::from("ls dir/a"),
                },
                CompletionChoice {
                    label: String::from("dir/b"),
                    completion: String::from("ls dir/b"),
                },
            ];
            assert_eq!(results, expected_results,);

            //TODO: always cleanup
            fs::remove_dir_all("/tmp/manette").unwrap();
        }

        #[test]
        fn test_autocomplete_path_absolute() {
            //TODO: use random dir name
            let test_dir = String::from("/tmp/manette-sub/test");
            fs::create_dir_all(test_dir.clone()).unwrap();
            fs::write(format!("{}/a", test_dir), "").unwrap();
            fs::write(format!("{}/b", test_dir), "").unwrap();
            let test_args = CommandArguments {
                command: String::from("ls"),
                arguments: vec![format!("{}/", test_dir)],
            };
            let mut results = autocomplete_path(test_args.clone(), None).unwrap();
            results.sort();
            let expected_results = vec![
                CompletionChoice {
                    label: format!("{}/a", test_dir),
                    completion: format!("ls {}/a", test_dir),
                },
                CompletionChoice {
                    label: format!("{}/b", test_dir),
                    completion: format!("ls {}/b", test_dir),
                },
            ];
            assert_eq!(results, expected_results,);

            //TODO: always cleanup
            fs::remove_dir_all("/tmp/manette-sub").unwrap();
        }

        #[test]
        fn test_autocomplete_path_root() {
            let test_args = CommandArguments {
                command: String::from("ls"),
                arguments: vec![format!("/tm")],
            };
            let mut results = autocomplete_path(test_args.clone(), None).unwrap();
            results.sort();
            let expected_results = vec![CompletionChoice {
                label: format!("/tmp/"),
                completion: format!("ls /tmp/"),
            }];
            assert_eq!(results, expected_results,);
        }

        #[test]
        fn test_directory_from_path() {
            assert_eq!(
                directory_from_path(&String::from("a/b/c/d")),
                Some(String::from("a/b/c/"))
            );
            assert_eq!(directory_from_path(&String::from("abcd")), None);
        }

        #[test]
        fn test_path_to_name() {
            let test_path = PathBuf::from("/tmp");
            assert_eq!(path_to_name(&test_path).unwrap(), String::from("tmp/"));
        }
    }
}
