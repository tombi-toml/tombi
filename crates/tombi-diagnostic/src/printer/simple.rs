use nu_ansi_term::Style;

use crate::{Diagnostic, Level, Print};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Simple {
    pub use_ansi_color: bool,
}

impl std::default::Default for Simple {
    fn default() -> Self {
        Self {
            use_ansi_color: true,
        }
    }
}

impl Print<Simple> for Level {
    fn print(&self, printer: &mut Simple) {
        let level_style = if printer.use_ansi_color {
            self.color().bold()
        } else {
            Style::new()
        };

        print!("{}", level_style.paint(self.as_padded_str()));
    }
}

impl Print<Simple> for Diagnostic {
    fn print(&self, printer: &mut Simple) {
        let message_style = if printer.use_ansi_color {
            Style::new().bold()
        } else {
            Style::new()
        };

        self.level().print(printer);
        println!(": {}", message_style.paint(self.message()));
    }
}
