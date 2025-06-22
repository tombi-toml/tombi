#[derive(thiserror::Error, Debug)]
pub enum ErrorKind {}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub range: tombi_text::Range,
}

impl Error {
    #[inline]
    pub fn code(&self) -> &'static str {
        match self.kind {}
    }
}

impl tombi_diagnostic::SetDiagnostics for Error {
    fn set_diagnostics(self, diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>) {
        diagnostics.push(tombi_diagnostic::Diagnostic::new_error(
            self.kind.to_string(),
            self.code(),
            self.range,
        ))
    }
}
