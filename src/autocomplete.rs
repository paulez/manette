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
    use anyhow::{Error, Result};
    use std::os::unix::fs::PermissionsExt;
    use crate::autocomplete::CompletionChoice;
    use crate::userenv::userenv;
    use std::ffi::OsString;
    use std::{env, error, io, fmt, fs};

    enum CompletionType {
        Executable,
        File,
    }

    #[derive(Clone, Debug)]
    struct CommandArguments {
        command: String,
        arguments: Vec<String>
    }


    impl fmt::Display for CommandArguments {
        fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
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
                AutoCompleteError::NonUnicodePath(ref err) => write!(f, "Invalid Unicode path"),
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
        let mut choices: Vec<CompletionChoice> = Vec::new();
        let command_args = build_command_arguments(command);
        match get_completion_type(&command_args) {
            CompletionType::File => {
                choices.append(&mut autocomplete_path(command_args)?);
            },
            CompletionType::Executable => {
                for path in userenv::path().split(":") {
                    choices.append(&mut executables_in_path_with_prefix(path, &command_args.command));
                }
            }
        }
        choices.sort();
        choices.dedup();
        Ok(choices)
    }

    fn get_completion_type(command_args: &CommandArguments) -> CompletionType {
        if command_args.arguments.len() > 0 {
            CompletionType::File
        } else {
            CompletionType::Executable
        }
    }

    fn build_command_arguments(input_command: &str) -> CommandArguments {
        let items: Vec<&str> = input_command.split(" ").collect();
        let mut arguments: Vec<String> = Vec::new();
        let mut command: String = String::new();
        for (i, item) in items.iter().enumerate() {
            if i == 0 {
                command = item.to_string();
            } else {
                arguments.push(item.to_string());
            }
        }
        CommandArguments {
            command,
            arguments
        }
    }

    fn executables_in_path_with_prefix(path: &str, prefix: &str) -> Vec<CompletionChoice> {
        match fs::read_dir(path) {
            Ok(paths) => {
                let mut path_strings: Vec<CompletionChoice> = Vec::new();

                for entry in paths {
                    match entry {
                        Ok(entry) => {
                            let filename = entry.file_name();
                            let filenamestr = filename.to_str();
                            match filenamestr {
                                Some(filenamestr) => {
                                    let filenamestring = filenamestr.to_string();
                                    if filenamestring.starts_with(prefix) {
                                        log::debug!("Testing entry metadata: {:?}", entry);
                                        let metadata = entry.metadata();
                                        match metadata {
                                            Ok(metadata) => {
                                                let permissions = metadata.permissions();
                                                if permissions.mode() & 0o111 != 0 {
                                                    let completion = CompletionChoice {
                                                        label: filenamestring.clone(),
                                                        completion: filenamestring
                                                    };
                                                    path_strings.push(completion);
                                                }
                                            }
                                            Err(_err) => {
                                                log::warn!("Cannot read metadata for {:?}", entry);
                                            }
                                        }
                                    }
                                }
                                None => {
                                    log::warn!("Cannot convert into str: {:?}", entry);
                                }
                            }
                        }
                        Err(ref _err) => {
                            log::warn!("Cannot read entry {:?}", entry);
                        }
                    }
                }
                path_strings
            }
            Err(_err) => {
                log::warn!("Cannot read {:?}", path);
                vec![]
            }
        }
    }


    fn autocomplete_path(command_args: CommandArguments) -> Result<Vec<CompletionChoice>> {
        log::debug!("Autocompleting path: {:?}", command_args);
        let empty_arg = String::from("");
        let current_arg = match command_args.arguments.last().clone() {
            Some(arg) => arg,
            None => &empty_arg,
        };
        let current_dir = env::current_dir()?;
        let paths = fs::read_dir(current_dir)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;
        let paths = paths
            .iter()
            .filter_map(|p| p.file_name())
            .filter_map(|name| name.to_str())
            .collect::<Vec<&str>>();
        let completions = paths
            .iter()
            .filter(|p| p.starts_with(current_arg))
            .map(|p| CompletionChoice{
                label: p.to_string(),
                completion: path_full_completion(command_args.clone(), p.to_string())
            })
            .collect::<Vec<CompletionChoice>>();
        Ok(completions)
    }

    fn path_full_completion(mut args: CommandArguments, completion: String) -> String {
        if args.arguments.len() > 0 {
            args.arguments.pop();
        }
        args.arguments.push(completion);
        args.to_string()
    }
}
