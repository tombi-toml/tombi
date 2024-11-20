mod arg;
mod command;

use clap::{
    builder::styling::{AnsiColor, Color, Style},
    Parser,
};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use tracing_subscriber::prelude::*;

/// Tombi: TOML formatter and linter.
#[derive(clap::Parser)]
#[command(name="tombi", version, styles=app_styles(), disable_help_subcommand(true))]
pub struct Args {
    #[command(subcommand)]
    pub subcommand: command::TomlCommand,
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}

impl<I, T> From<I> for Args
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    #[inline]
    fn from(value: I) -> Self {
        Self::parse_from(value)
    }
}

pub fn run(args: impl Into<Args>) -> Result<(), crate::Error> {
    let args: Args = args.into();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from(
            args.verbose.log_level_filter().to_string(),
        ))
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_writer(std::io::stderr)
                .without_time(),
        )
        .init();

    match args.subcommand {
        command::TomlCommand::Format(args) => command::format::run(args),
        command::TomlCommand::Lint(args) => command::lint::run(args),
    }
}

fn app_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Blue))),
        )
        .header(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Blue))),
        )
        .literal(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Cyan))),
        )
        .valid(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Green))),
        )
        .invalid(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .error(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan))))
}
