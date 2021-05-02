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
