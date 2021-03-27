use std::error::Error;
use cursive::{Cursive, CursiveExt};
use cursive::views::{DummyView, LinearLayout, Panel, EditView, TextView, ResizedView};
use cursive_core::view::Nameable;
use cursive_flexi_logger_view::FlexiLoggerView;
use flexi_logger::{Logger, LogTarget};

mod command;

use crate::command::run;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
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

    siv.add_layer(
        Panel::new(
        LinearLayout::vertical()
                .child(EditView::new()
                       .on_submit(user_input)
                       .with_name("command_input")
                )
                .child(DummyView)
                .child(ResizedView::with_full_screen(
                    TextView::new("Command output")
                        .with_name("command_output")
                ))
                .child(TextView::new("Command error")
                    .with_name("command_error"))
                .child(FlexiLoggerView::scrollable())
        )
            .title("manette")
    );
    siv.add_global_callback('q', |s| s.quit());
    log::info!("test log message");
    siv.run();
    Ok(())
}

fn user_input(s: &mut Cursive, command: &str) {
    let command_output = run::run_command(command);
    let stdout = String::from_utf8(command_output.stdout).unwrap();
    let stderr = String::from_utf8(command_output.stderr).unwrap();
    s.call_on_name("command_output", |view: &mut TextView| {
        view.set_content(stdout);
    });
    s.call_on_name("command_error", |view: &mut TextView| {
        view.set_content(stderr);
    });
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
