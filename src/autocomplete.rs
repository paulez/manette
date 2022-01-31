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

pub mod autocomplete {
    use std::os::unix::fs::PermissionsExt;
use crate::userenv::userenv;
    use std::fs;

    pub fn autocomplete(command: &str) -> Vec<String> {
        let mut choices: Vec<String> = Vec::new();
        for path in userenv::path().split(":") {
            choices.append(&mut executables_in_path_with_prefix(path, command));
        }
        choices.sort();
        choices.dedup();
        choices
    }

    fn executables_in_path_with_prefix(path: &str, prefix: &str) -> Vec<String> {
        match fs::read_dir(path) {
            Ok(paths) => {
                let mut path_strings: Vec<String> = Vec::new();

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
                                                    path_strings.push(filenamestring);
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
}
