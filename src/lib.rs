use std::{error::Error, path::PathBuf, env};
use cursive::{Cursive, CursiveExt};
use cursive::theme::{Color, PaletteColor, Theme};
use cursive::views::{DummyView, LinearLayout, Panel, EditView, TextView, ResizedView, ScrollView};
use cursive_core::view::Nameable;
use cursive_flexi_logger_view::FlexiLoggerView;
use flexi_logger::{Logger, LogTarget};

mod command;

use crate::command::run;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {

    let run_state = RunState::new();
    let user_input= move |s: &mut Cursive, command: &str| {
        let command_output = run::run_command(command, &run_state);
        let stdout: String;
        let stderr: String;
        match command_output {
            Ok(output) => {
                stdout = String::from_utf8(output.stdout).unwrap();
                stderr = String::from_utf8(output.stderr).unwrap();
            }
            Err(output) => {
                stdout = String::from("");
                stderr = output.to_string();
            }
        }
        s.call_on_name("command_output", |view: &mut TextView| {
            view.set_content(stdout);
        });
        s.call_on_name("command_error", |view: &mut TextView| {
            view.set_content(stderr);
        });
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
    siv.add_layer(
        Panel::new(
        LinearLayout::vertical()
                .child(EditView::new()
                       .on_submit(user_input)
                       .with_name("command_input")
                )
                .child(DummyView)
                .child(ResizedView::with_full_screen(
                    ScrollView::new(
                    TextView::new("Command output")
                        .with_name("command_output")
                )))
                .child(TextView::new("Command error")
                    .with_name("command_error"))
                .child(FlexiLoggerView::scrollable())
        )
            .title("manette")
    );
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
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() > 1 {
            return Err("No supported argument!");
        }
        Ok(Config { })
    }
}

pub struct RunState {
    current_dir: PathBuf,
}

impl RunState {
    fn new() -> RunState {
        let current_dir: PathBuf = env::current_dir().unwrap();
        RunState {current_dir}
    }
}
