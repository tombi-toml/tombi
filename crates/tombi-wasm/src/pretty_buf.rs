use std::fmt;
use std::sync::{Arc, Mutex};
use nu_ansi_term::{Color, Style};
use tombi_diagnostic::{Diagnostic, Level, Print};
use tombi_diagnostic::printer::Simple;
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct PrettyBuf(Arc<Mutex<String>>);

impl Default for PrettyBuf {
    fn default() -> Self {
        Self::new()
    }
}

impl PrettyBuf {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(String::new())))
    }

    pub fn get(&self) -> String {
        self.0.lock().unwrap().clone()
    }
}

impl Write for PrettyBuf {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut guard = self.0.lock().unwrap();
        guard.push_str(s);
        Ok(())
    }
}

impl Print<PrettyBuf> for Level {
    fn print(&self, _printer: &mut PrettyBuf) {
        self.print(&mut Simple);
    }
}

impl Print<PrettyBuf> for Diagnostic {
    fn print(&self, printer: &mut PrettyBuf) {
        self.level().print(printer);
        writeln!(printer, ": {}", Style::new().bold().paint(self.message())).unwrap();

        let at_style: Style = Style::new().fg(Color::DarkGray);
        let link_style: Style = Style::new().fg(Color::Cyan);
        if let Some(source_file) = self.source_file() {
            writeln!(
                printer,
                "    {} {}",
                at_style.paint("at"),
                link_style.paint(format!(
                    "{}:{}:{}",
                    source_file.display(),
                    self.position().line() + 1,
                    self.position().column() + 1
                )),
            )
            .unwrap();
        } else {
            writeln!(
                printer,
                "    {}",
                at_style.paint(format!(
                    "at line {} column {}",
                    self.position().line() + 1,
                    self.position().column() + 1
                )),
            )
            .unwrap();
        }
    }
}
