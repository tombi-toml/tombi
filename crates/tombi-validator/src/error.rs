use std::cmp::Ordering;

pub const TYPE_MATCHED_SCORE: u8 = 1;
pub const REQUIRED_KEY_SCORE: u8 = 1;

/// # Validation Error
///
/// The `score` field is used to indicate how well an error matches the schema.
/// When multiple errors are possible, the error with the highest score is returned.
/// This helps to filter out unnecessary error messages and provide the most relevant feedback to the user.
/// For example, a higher score means the validation matched more required keys or types in the schema.
///
#[derive(Debug)]
pub struct Error {
    pub score: u8,
    pub diagnostics: Vec<tombi_diagnostic::Diagnostic>,
    pub evaluated_locations: crate::EvaluatedLocations,
}

impl Default for Error {
    fn default() -> Self {
        Self::new()
    }
}

impl Error {
    #[inline]
    pub fn new() -> Self {
        Self {
            score: 0,
            diagnostics: vec![],
            evaluated_locations: Default::default(),
        }
    }

    #[inline]
    pub fn combine(&mut self, mut other: Self) {
        let mut merged_locations = std::mem::take(&mut self.evaluated_locations);
        merged_locations.merge_from(std::mem::take(&mut other.evaluated_locations));

        match self.score.cmp(&other.score) {
            Ordering::Greater => {}
            Ordering::Less => std::mem::swap(self, &mut other),
            Ordering::Equal => {
                self.diagnostics.extend(other.diagnostics);
            }
        }

        self.evaluated_locations = merged_locations;
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
            score: TYPE_MATCHED_SCORE,
            diagnostics,
            evaluated_locations: Default::default(),
        }
    }
}
