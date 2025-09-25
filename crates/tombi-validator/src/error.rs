use std::cmp::Ordering;

#[derive(Debug)]
pub struct Error {
    pub score: u8,
    pub diagnostics: Vec<tombi_diagnostic::Diagnostic>,
}

impl Error {
    #[inline]
    pub fn new() -> Self {
        Self {
            score: 0,
            diagnostics: vec![],
        }
    }

    #[inline]
    pub fn combine(&mut self, mut other: Self) {
        match self.score.cmp(&other.score) {
            Ordering::Greater => {}
            Ordering::Less => std::mem::swap(self, &mut other),
            Ordering::Equal => {
                self.diagnostics.extend(other.diagnostics);
            }
        }
    }

    #[inline]
    pub fn prepend_diagnostics(&mut self, mut other: Vec<tombi_diagnostic::Diagnostic>) {
        std::mem::swap(&mut self.diagnostics, &mut other);
        self.diagnostics.extend(other);
    }
}

impl From<Vec<tombi_diagnostic::Diagnostic>> for Error {
    fn from(diagnostics: Vec<tombi_diagnostic::Diagnostic>) -> Self {
        Self {
            score: 1, // Type matched points.
            diagnostics,
        }
    }
}
