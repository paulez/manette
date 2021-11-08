/*

Copyright or © or Copr. Paul Ezvan (2021)

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

pub mod open {
    use cursive::Cursive;
    use std::process::Command;

    use crate::command::run;
    use crate::ui::update;

    pub fn open_file(s: &mut Cursive, filename: &str) {
        let output = Command::new("file").arg("-b").arg("--mime-type").arg(filename).output();
        match output {
            Ok(output) => {
                let mut output = String::from_utf8(output.stdout).unwrap();
                if output.ends_with('\n') {
                    output.pop();
                    if output.ends_with('\r') {
                        output.pop();
                    }
                }
                log::debug!("file output is {}", output);
                match output.as_str() {
                    "text/plain" => {
                        log::debug!("File is text");
                        run::run_command(format!("cat {}", filename).as_str(), s);
                    }
                    _ => {
                        log::debug!("File is something else");
                    }
                };
            }
            Err(error) => {
                log::error!("Failed to change dir {:?}", error);
                update::show_error(s, format!("Failed to change dir {:?}", error))
            }
        }
    }
}

pub mod filetype {

    use std::fs::{DirEntry, Metadata};
    use std::os::unix::prelude::PermissionsExt;

    pub fn get_type(dir_entry: DirEntry) -> FileType {
        let metadata = dir_entry.metadata();
        let filetype: FileType = match metadata {
            Ok(metadata) => {
                get_type_from_metadata(metadata)
            }
            Err(error) => {
                log::error!("Cannot get metadata: {:?}", error);
                FileType::Unknown
            }
        };
        filetype
    }

    pub fn get_type_from_metadata(metadata: Metadata) -> FileType {
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

    pub enum FileType {
        Directory,
        Executable,
        File,
        Symlink,
        Unknown,
    }
}
