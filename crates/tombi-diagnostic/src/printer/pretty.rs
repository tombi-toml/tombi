use nu_ansi_term::{Color, Style};

use crate::{printer::Simple, Diagnostic, Level, Print};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pretty;

impl Print<Pretty> for Level {
    fn print(&self, _printer: &mut Pretty) {
        self.print(&mut Simple);
    }
}

impl Print<Pretty> for Diagnostic {
    fn print(&self, printer: &mut Pretty) {
        self.level().print(printer);
        println!(": {}", Style::new().bold().paint(self.message()));

        let at_style: Style = Style::new().fg(Color::DarkGray);
        let link_style: Style = Style::new().fg(Color::Cyan);
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
