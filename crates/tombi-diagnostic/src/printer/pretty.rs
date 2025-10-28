use nu_ansi_term::{Color, Style};

use crate::{printer::Simple, Diagnostic, Level, Print};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pretty;

impl Print<Pretty> for Level {
    fn print(&self, _printer: &mut Pretty, use_ansi_color: bool) {
        self.print(&mut Simple, use_ansi_color);
    }
}

impl Print<Pretty> for Diagnostic {
    fn print(&self, printer: &mut Pretty, use_ansi_color: bool) {
        self.level().print(printer, use_ansi_color);

        let (message_style, at_style, link_style) = if use_ansi_color {
            (
                Style::new().bold(),
                Style::new().fg(Color::DarkGray),
                Style::new().fg(Color::Cyan),
            )
        } else {
            (Style::new(), Style::new(), Style::new())
        };

        println!(": {}", message_style.paint(self.message()));

        if let Some(source_file) = self.source_file() {
            println!(
                "    {} {}",
                at_style.paint("at"),
                link_style.paint(format!(
                    "{}:{}:{}",
                    source_file.display(),
                    self.position().line + 1,
                    self.position().column + 1
                )),
            );
        } else {
            println!(
                "    {}",
                at_style.paint(format!(
                    "at line {} column {}",
                    self.position().line + 1,
                    self.position().column + 1
                )),
            );
        }
    }
}
