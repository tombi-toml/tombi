use env_logger::fmt::Formatter;
use log::Record;
use nu_ansi_term::Style;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

static HAS_WARNING: AtomicBool = AtomicBool::new(false);
static HAS_ERROR: AtomicBool = AtomicBool::new(false);

/// Whether a warning has been logged.
pub fn has_warning() -> bool {
    HAS_WARNING.load(Ordering::Relaxed)
}

/// Whether an error has been logged.
pub fn has_error() -> bool {
    HAS_ERROR.load(Ordering::Relaxed)
}

/// Formats the records, and remembers whether a warning or an error was logged,
/// so that a command can decide its exit code.
///
/// The records that are formatted are the ones that pass the filter, which always admits warnings.
/// `max_level` only decides which of them are written, so that `-qq` does not change the exit code.
pub fn format(
    max_level: log::LevelFilter,
    use_ansi_color: bool,
    show_trace_location: bool,
) -> impl Fn(&mut Formatter, &Record) -> std::io::Result<()> + Send + Sync + 'static {
    move |f, record| {
        match record.level() {
            log::Level::Error => HAS_ERROR.store(true, Ordering::Relaxed),
            log::Level::Warn => HAS_WARNING.store(true, Ordering::Relaxed),
            _ => {}
        }

        if record.level() > max_level {
            return Ok(());
        }

        let level_style = level_style_for(record.level(), use_ansi_color);
        let level_str = match record.level() {
            log::Level::Error => "Error",
            log::Level::Warn => "Warning",
            log::Level::Info => "Info",
            log::Level::Debug => "Debug",
            log::Level::Trace => "Trace",
        };

        write!(f, "{}: ", level_style.paint(format!("{:>7}", level_str)))?;

        writeln!(f, "{}", record.args())?;

        if show_trace_location && let (Some(file), Some(line)) = (record.file(), record.line()) {
            let link = format!("{file}:{line}");
            writeln!(
                f,
                "    {} {}",
                at_style(use_ansi_color).paint("at"),
                link_style(use_ansi_color).paint(link)
            )?;
        }

        Ok(())
    }
}

fn level_style_for(level: log::Level, use_ansi_color: bool) -> Style {
    if use_ansi_color {
        match level {
            log::Level::Error => Style::new().bold().fg(nu_ansi_term::Color::Red),
            log::Level::Warn => Style::new().bold().fg(nu_ansi_term::Color::Yellow),
            log::Level::Info => Style::new().bold().fg(nu_ansi_term::Color::Green),
            log::Level::Debug => Style::new().bold().fg(nu_ansi_term::Color::Blue),
            log::Level::Trace => Style::new().bold().fg(nu_ansi_term::Color::Magenta),
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
