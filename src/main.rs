use clap::{App, Arg};
use std::process;

use manette::Config;

fn main() {
    let matches = App::new("Manette")
        .version("0.1")
        .author("Paul Ezvan <paul@ezvan.fr>")
        .about("Terminal file explorer and command runner")
        .arg(
            Arg::with_name("debug")
                .long("debug")
                .short("d")
                .help("Enable debug"),
        )
        .get_matches();

    let config = Config::new(&matches).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = manette::run(config) {
        println!("Application error: {}", e);
        process::exit(1);
    }
    panic!("crash and burn");
}
