mod command;
mod tracing_formatter;

use clap_verbosity_flag::log;
use tracing_formatter::TombiFormatter;
use tracing_subscriber::prelude::*;

use crate::args::{Args, TomlCommand};

/// Convert [`clap_verbosity_flag::log::LevelFilter`] to [`tracing_subscriber::filter::LevelFilter`]
fn convert_log_level_filter(level: log::LevelFilter) -> tracing_subscriber::filter::LevelFilter {
    match level {
        log::LevelFilter::Off => tracing_subscriber::filter::LevelFilter::OFF,
        log::LevelFilter::Error => tracing_subscriber::filter::LevelFilter::ERROR,
        log::LevelFilter::Warn => tracing_subscriber::filter::LevelFilter::WARN,
        log::LevelFilter::Info => tracing_subscriber::filter::LevelFilter::INFO,
        log::LevelFilter::Debug => tracing_subscriber::filter::LevelFilter::DEBUG,
        log::LevelFilter::Trace => tracing_subscriber::filter::LevelFilter::TRACE,
    }
}

pub fn run(args: impl Into<Args>) -> Result<(), crate::Error> {
    let args: Args = args.into();
    tracing_subscriber::registry()
        .with(
            // Filter out all logs from other crates
            tracing_subscriber::filter::Targets::new()
                .with_target(
                    "tombi",
                    convert_log_level_filter(args.verbose.log_level_filter()),
                )
                .with_target(
                    "serde_tombi",
                    convert_log_level_filter(args.verbose.log_level_filter()),
                )
                .with_default(tracing_subscriber::filter::LevelFilter::OFF),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(TombiFormatter::from(args.verbose.log_level_filter()))
                .with_writer(std::io::stderr),
        )
        .init();

    match args.subcommand {
        TomlCommand::Format(args) => command::format::run(args),
        TomlCommand::Lint(args) => command::lint::run(args),
        TomlCommand::Lsp(args) => command::lsp::run(args),
        TomlCommand::Completion(args) => command::completion::run(args),
    }
}
