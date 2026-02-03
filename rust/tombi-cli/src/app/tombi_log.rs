use env_logger::fmt::Formatter;
use log::Record;
use nu_ansi_term::Style;
use std::io::Write;

pub fn format(
    use_ansi_color: bool,
    show_trace_location: bool,
) -> impl Fn(&mut Formatter, &Record) -> std::io::Result<()> + Send + Sync + 'static {
    move |f, record| {
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

        if show_trace_location {
            if let (Some(file), Some(line)) = (record.file(), record.line()) {
                let link = format!("{file}:{line}");
                writeln!(
                    f,
                    "    {} {}",
                    at_style(use_ansi_color).paint("at"),
                    link_style(use_ansi_color).paint(link)
                )?;
            }
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
