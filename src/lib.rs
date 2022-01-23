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

use clap::ArgMatches;
use cursive::theme::{Color, PaletteColor, Theme};
use cursive::views::{DummyView, LinearLayout, Panel};
use cursive::{Cursive, CursiveExt};
use cursive_core::view::Nameable;
use cursive_flexi_logger_view::FlexiLoggerView;
use flexi_logger::{LogTarget, Logger};
use std::{env, error::Error, path::PathBuf};

mod command;
mod file;
mod ui;
mod userenv;
mod view;

use crate::command::run;
use crate::view::CliView;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let user_input = move |s: &mut Cursive, command: &str| {
        run::run_command(command, s);
        s.call_on_name("cli_input", |view: &mut CliView| {
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
        .child(CliView::new().on_submit(user_input).with_name("cli_input"))
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
