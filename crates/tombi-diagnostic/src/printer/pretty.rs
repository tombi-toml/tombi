use nu_ansi_term::{Color, Style};

use crate::{Diagnostic, Level, Print, printer::Simple};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pretty {
    pub use_ansi_color: bool,
}

impl std::default::Default for Pretty {
    fn default() -> Self {
        Self {
            use_ansi_color: true,
        }
    }
}

impl Print<Pretty> for Level {
    fn print(&self, printer: &mut Pretty) {
        self.print(&mut Simple {
            use_ansi_color: printer.use_ansi_color,
        });
    }
}

impl Print<Pretty> for Diagnostic {
    fn print(&self, printer: &mut Pretty) {
        self.level().print(printer);

        let (message_style, at_style, link_style) = if printer.use_ansi_color {
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
