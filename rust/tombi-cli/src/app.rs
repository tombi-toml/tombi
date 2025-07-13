mod arg;
mod command;
mod tracing_formatter;

use clap::{
    builder::styling::{AnsiColor, Color, Style},
    Parser,
};
use clap_verbosity_flag::{log, InfoLevel, Verbosity};
use tracing_formatter::TombiFormatter;
use tracing_subscriber::filter;
use tracing_subscriber::prelude::*;

#[derive(clap::Parser)]
#[command(
    name="tombi",
    about = app_about(),
    version = env!("__TOMBI_VERSION"),
    styles=app_styles(),
    disable_help_subcommand(true),
)]
pub struct Args {
    #[command(subcommand)]
    pub subcommand: command::TomlCommand,

    /// Do not fetch from remote
    #[clap(long, global = true)]
    offline: bool,

    /// Do not use cache
    ///
    /// Fetch the latest data from remote and save it to the cache
    #[clap(long, global = true, env("TOMBI_NO_CACHE"))]
    no_cache: bool,

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

/// Convert [`clap_verbosity_flag::log::LevelFilter`] to [`tracing_subscriber::filter::LevelFilter`]
fn convert_log_level_filter(level: log::LevelFilter) -> filter::LevelFilter {
    match level {
        log::LevelFilter::Off => filter::LevelFilter::OFF,
        log::LevelFilter::Error => filter::LevelFilter::ERROR,
        log::LevelFilter::Warn => filter::LevelFilter::WARN,
        log::LevelFilter::Info => filter::LevelFilter::INFO,
        log::LevelFilter::Debug => filter::LevelFilter::DEBUG,
        log::LevelFilter::Trace => filter::LevelFilter::TRACE,
    }
}

pub fn run(args: impl Into<Args>) -> Result<(), crate::Error> {
    let args: Args = args.into();
    tracing_subscriber::registry()
        .with(
            // Filter out all logs from other crates
            filter::Targets::new()
                .with_target(
                    "tombi",
                    convert_log_level_filter(args.verbose.log_level_filter()),
                )
                .with_default(filter::LevelFilter::OFF),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(TombiFormatter::from(args.verbose.log_level_filter()))
                .with_writer(std::io::stderr),
        )
        .init();

    let offline = args.offline;
    let no_cache = args.no_cache;
    match args.subcommand {
        command::TomlCommand::Format(args) => command::format::run(args, offline, no_cache),
        command::TomlCommand::Lint(args) => command::lint::run(args, offline, no_cache),
        command::TomlCommand::Lsp(args) => command::lsp::run(args, offline, no_cache),
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
        "{title_style}              {title} {title_style:#}{desc_style}: TOML Toolkit              {desc_style:#}"
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
