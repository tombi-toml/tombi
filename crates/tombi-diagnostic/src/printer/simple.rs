use nu_ansi_term::Style;

use crate::{Diagnostic, Level, Print};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Simple;

impl Print<Simple> for Level {
    fn print(&self, _printer: &mut Simple, use_ansi_color: bool) {
        let level_style = if use_ansi_color {
            self.color().bold()
        } else {
            Style::new()
        };

        print!("{}", level_style.paint(self.as_padded_str()));
    }
}

impl Print<Simple> for Diagnostic {
    fn print(&self, printer: &mut Simple, use_ansi_color: bool) {
        let message_style = if use_ansi_color {
            Style::new().bold()
        } else {
            Style::new()
        };

        self.level().print(printer, use_ansi_color);
        println!(": {}", message_style.paint(self.message()));
    }
}
