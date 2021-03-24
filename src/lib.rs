use std::error::Error;
use cursive::{Cursive, CursiveExt};
use cursive::views::TextView;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut siv = Cursive::new();
    siv.add_layer(TextView::new("Hello World!\nPress q to quit."));
    siv.add_global_callback('q', |s| s.quit());
    siv.run();
    Ok(())
}

pub struct Config {
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() > 1 {
            return Err("No supported argument!");
        }
        Ok(Config { })
    }
}
