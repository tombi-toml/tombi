use itertools::Itertools;

#[derive(thiserror::Error, Debug)]
pub enum ErrorKind {}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub range: tombi_text::Range,
}

impl tombi_diagnostic::SetDiagnostics for Error {
    fn set_diagnostics(self, diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>) {
        diagnostics.push(tombi_diagnostic::Diagnostic::new_error(
            self.kind.to_string(),
            self.range,
        ))
    }
}

#[derive(Debug)]
pub struct Patterns(pub Vec<String>);

impl std::fmt::Display for Patterns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.len() == 1 {
            write!(f, "{}", self.0[0])
        } else {
            write!(f, "{}", self.0.iter().map(|p| format!("({})", p)).join("|"))
        }
    }
}
