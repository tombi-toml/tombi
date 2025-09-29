mod command;
mod options;
mod tracing_formatter;

use clap::{
    builder::styling::{AnsiColor, Color, Style},
    Parser,
};
use tracing_formatter::TombiFormatter;
use tracing_subscriber::prelude::*;

use crate::app::options::Verbosity;

#[derive(clap::Parser)]
#[command(
    name="tombi",
    about = app_about(),
    version = env!("__TOMBI_VERSION").trim_start_matches('v'),
    styles=app_styles(),
    disable_help_subcommand(true),
)]
pub struct Args {
    #[command(subcommand)]
    pub subcommand: command::TomlCommand,

    #[command(flatten)]
    verbose: Verbosity,
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

#[derive(clap::Args, Debug)]
struct CommonArgs {
    /// Disable network access
    ///
    /// Don't fetch from remote and use local schemas cache.
    #[clap(long, global = true, env("TOMBI_OFFLINE"))]
    offline: bool,

    /// Do not use cache
    ///
    /// Fetch the latest data from remote and save it to the cache
    #[clap(long, global = true, env("TOMBI_NO_CACHE"))]
    no_cache: bool,
}

pub fn run(args: impl Into<Args>) -> Result<(), crate::Error> {
    let args: Args = args.into();
    let log_level_filter = args.verbose.log_level_filter();
    tracing_subscriber::registry()
        .with(
            // Filter out all logs from other crates
            tracing_subscriber::filter::Targets::new()
                .with_target("tombi", log_level_filter)
                .with_target("serde_tombi", log_level_filter)
                .with_default(tracing_subscriber::filter::LevelFilter::OFF),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(TombiFormatter::from(log_level_filter))
                .with_writer(std::io::stderr),
        )
        .init();

    match args.subcommand {
        command::TomlCommand::Format(args) => command::format::run(args),
        command::TomlCommand::Lint(args) => command::lint::run(args),
        command::TomlCommand::Lsp(args) => command::lsp::run(args),
        command::TomlCommand::Completion(args) => command::completion::run(args),
    }
}

fn app_about() -> String {
    let title = "Tombi";
    let title_style = Style::new()
        .bold()
        .bg_color(Some(Color::Ansi(AnsiColor::Blue)))
        .fg_color(Some(Color::Ansi(AnsiColor::White)));

    let desc_style = Style::new()
        .bg_color(Some(Color::Ansi(AnsiColor::Blue)))
        .fg_color(Some(Color::Ansi(AnsiColor::White)));

    format!(
        "{title_style}                          {title} {title_style:#}{desc_style}: TOML Toolkit                          {desc_style:#}"
    )
}

const fn app_styles() -> clap::builder::Styles {
    clap::builder::Styles::plain()
        .header(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Blue))),
        )
        .error(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .usage(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Blue))),
        )
        .literal(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Cyan))),
        )
        .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan))))
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
}
