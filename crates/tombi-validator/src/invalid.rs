#[derive(Debug)]
pub struct Invalid {
    /// Whether a JSON Schema assertion failed independently of diagnostic severity.
    pub assertion_failed: bool,
    pub match_evidence: Box<crate::MatchEvidence>,
    pub diagnostics: Vec<tombi_diagnostic::Diagnostic>,
    pub local_evaluated_locations: crate::Valid,
}

impl Default for Invalid {
    fn default() -> Self {
        Self::new()
    }
}

impl Invalid {
    #[inline]
    pub fn new() -> Self {
        Self {
            assertion_failed: false,
            match_evidence: Default::default(),
            diagnostics: vec![],
            local_evaluated_locations: Default::default(),
        }
    }

    #[inline]
    pub fn combine(&mut self, mut other: Self) {
        let accumulator_is_empty = !self.assertion_failed
            && self.diagnostics.is_empty()
            && *self.match_evidence == crate::MatchEvidence::default();
        let ordering = self
            .match_evidence
            .score()
            .cmp(&other.match_evidence.score());
        if accumulator_is_empty || ordering.is_lt() {
            std::mem::swap(self, &mut other);
        } else if ordering.is_eq() {
            self.assertion_failed |= other.assertion_failed;
            self.diagnostics.extend(other.diagnostics);
            self.local_evaluated_locations
                .merge_from(other.local_evaluated_locations);
        }
    }

    #[inline]
    pub fn prepend_diagnostics(&mut self, mut other: Vec<tombi_diagnostic::Diagnostic>) {
        std::mem::swap(&mut self.diagnostics, &mut other);
        self.diagnostics.extend(other);
    }
}

impl From<Vec<tombi_diagnostic::Diagnostic>> for Invalid {
    fn from(diagnostics: Vec<tombi_diagnostic::Diagnostic>) -> Self {
        Self {
            assertion_failed: diagnostics
                .iter()
                .any(|diagnostic| diagnostic.level() == tombi_diagnostic::Level::ERROR),
            match_evidence: Default::default(),
            diagnostics,
            local_evaluated_locations: Default::default(),
        }
    }
}
