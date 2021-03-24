use std::error::Error;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
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
