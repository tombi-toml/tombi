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
    fn level_style_for(level: &tracing::Level, use_ansi_color: bool) -> Style {
        if use_ansi_color {
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

    fn at_style(use_ansi_color: bool) -> Style {
        if use_ansi_color {
            Style::new().fg(nu_ansi_term::Color::DarkGray)
        } else {
            Style::new()
        }
    }

    fn link_style(use_ansi_color: bool) -> Style {
        if use_ansi_color {
            Style::new().fg(nu_ansi_term::Color::Cyan)
        } else {
            Style::new()
        }
    }
}

impl From<tracing_subscriber::filter::LevelFilter> for TombiFormatter {
    fn from(level: tracing_subscriber::filter::LevelFilter) -> Self {
        let level = match level {
            tracing_subscriber::filter::LevelFilter::OFF => None,
            tracing_subscriber::filter::LevelFilter::ERROR => Some(tracing::Level::ERROR),
            tracing_subscriber::filter::LevelFilter::WARN => Some(tracing::Level::WARN),
            tracing_subscriber::filter::LevelFilter::INFO => Some(tracing::Level::INFO),
            tracing_subscriber::filter::LevelFilter::DEBUG => Some(tracing::Level::DEBUG),
            tracing_subscriber::filter::LevelFilter::TRACE => Some(tracing::Level::TRACE),
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
        let use_ansi_color = std::env::var("NO_COLOR").map_or(true, |v| v.is_empty());
        let metadata = event.metadata();

        write!(
            writer,
            "{}: ",
            Self::level_style_for(metadata.level(), use_ansi_color).paint(format!(
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
                    Self::at_style(use_ansi_color).paint("at"),
                    Self::link_style(use_ansi_color).paint(link)
                )?;
            }
        }

        Ok(())
    }
}
