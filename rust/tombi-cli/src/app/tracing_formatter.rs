use nu_ansi_term::Style;
use tracing::{Event, Subscriber};
use tracing_subscriber::{
    fmt::{FmtContext, FormatEvent, FormatFields},
    registry::LookupSpan,
};

#[derive(Debug)]
pub struct TombiFormatter {
    level: Option<tracing::Level>,
}

impl TombiFormatter {
    fn level_style_for(level: &tracing::Level, ansi: bool) -> Style {
        if ansi {
            match *level {
                tracing::Level::ERROR => Style::new().bold().fg(nu_ansi_term::Color::Red),
                tracing::Level::WARN => Style::new().bold().fg(nu_ansi_term::Color::Yellow),
                tracing::Level::INFO => Style::new().bold().fg(nu_ansi_term::Color::Green),
                tracing::Level::DEBUG => Style::new().bold().fg(nu_ansi_term::Color::Blue),
                tracing::Level::TRACE => Style::new().bold().fg(nu_ansi_term::Color::Magenta),
            }
            .bold()
        } else {
            Style::new()
        }
    }

    fn at_style(ansi: bool) -> Style {
        if ansi {
            Style::new().fg(nu_ansi_term::Color::DarkGray)
        } else {
            Style::new()
        }
    }

    fn link_style(ansi: bool) -> Style {
        if ansi {
            Style::new().fg(nu_ansi_term::Color::Cyan)
        } else {
            Style::new()
        }
    }
}

impl From<clap_verbosity_flag::log::LevelFilter> for TombiFormatter {
    fn from(level: clap_verbosity_flag::log::LevelFilter) -> Self {
        let level = match level {
            clap_verbosity_flag::log::LevelFilter::Off => None,
            clap_verbosity_flag::log::LevelFilter::Error => Some(tracing::Level::ERROR),
            clap_verbosity_flag::log::LevelFilter::Warn => Some(tracing::Level::WARN),
            clap_verbosity_flag::log::LevelFilter::Info => Some(tracing::Level::INFO),
            clap_verbosity_flag::log::LevelFilter::Debug => Some(tracing::Level::DEBUG),
            clap_verbosity_flag::log::LevelFilter::Trace => Some(tracing::Level::TRACE),
        };

        Self { level }
    }
}

impl<S, N> FormatEvent<S, N> for TombiFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: tracing_subscriber::fmt::format::Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        let ansi = std::env::var("NO_COLOR").map_or(true, |v| v.is_empty());
        let metadata = event.metadata();

        write!(
            writer,
            "{}: ",
            Self::level_style_for(metadata.level(), ansi).paint(format!(
                "{:>7}",
                match *metadata.level() {
                    tracing::Level::ERROR => "Error",
                    tracing::Level::WARN => "Warning",
                    tracing::Level::INFO => "Info",
                    tracing::Level::DEBUG => "Debug",
                    tracing::Level::TRACE => "Trace",
                }
            ))
        )?;

        ctx.field_format().format_fields(writer.by_ref(), event)?;
        writeln!(writer)?;

        if self.level == Some(tracing::Level::TRACE) {
            if let Some(file) = metadata.file() {
                let link = if let Some(line) = metadata.line() {
                    format!("{file}:{line}")
                } else {
                    file.to_string()
                };

                writeln!(
                    writer,
                    "    {} {}",
                    Self::at_style(ansi).paint("at"),
                    Self::link_style(ansi).paint(link)
                )?;
            }
        }

        Ok(())
    }
}
