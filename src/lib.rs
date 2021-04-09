use clap::ArgMatches;
use cursive::theme::{Color, PaletteColor, Theme};
use cursive::views::{DummyView, EditView, LinearLayout, Panel};
use cursive::{Cursive, CursiveExt};
use cursive_core::view::Nameable;
use cursive_flexi_logger_view::FlexiLoggerView;
use flexi_logger::{LogTarget, Logger};
use std::{env, error::Error, path::PathBuf};

mod command;
mod ui;

use crate::command::run;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let user_input = move |s: &mut Cursive, command: &str| {
        run::run_command(command, s);
        s.call_on_name("command_input", |view: &mut EditView| {
            view.set_content("");
        });
    };

    let mut siv = Cursive::new();

    Logger::with_env_or_str("info, manette = debug")
        .log_target(LogTarget::FileAndWriter(
            cursive_flexi_logger_view::cursive_flexi_logger(&siv),
        ))
        .directory("logs")
        .suppress_timestamp()
        .format(flexi_logger::colored_with_thread)
        .start()
        .expect("failed to initialize logger!");

    let theme = custom_theme_from_cursive(&siv);
    siv.set_theme(theme);
    siv.add_global_callback('q', |s| s.quit());
    let mut layout = LinearLayout::vertical()
        .child(
            EditView::new()
                .on_submit(user_input)
                .with_name("command_input"),
        )
        .child(DummyView)
        .child(LinearLayout::vertical().with_name("command_layout"));
    if config.debug {
        layout.add_child(FlexiLoggerView::scrollable());
    }
    siv.add_layer(Panel::new(layout).title("manette"));
    command::run::run_command("ls", &mut siv);
    siv.run();
    Ok(())
}

fn custom_theme_from_cursive(siv: &Cursive) -> Theme {
    // We'll return the current theme with a small modification.
    let mut theme = siv.current_theme().clone();

    theme.palette[PaletteColor::Background] = Color::TerminalDefault;

    theme
}

pub struct Config {
    debug: bool,
}

impl Config {
    pub fn new(matches: &ArgMatches) -> Result<Config, &'static str> {
        let debug = matches.occurrences_of("debug") > 0;
        Ok(Config { debug })
    }
}

pub struct RunState {
    current_dir: PathBuf,
}

impl RunState {
    fn new() -> RunState {
        let current_dir: PathBuf = env::current_dir().unwrap();
        RunState { current_dir }
    }
}
