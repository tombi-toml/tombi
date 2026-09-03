#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Valid {
    pub properties: tombi_hashmap::IndexSet<String>,
    pub indices: tombi_hashmap::IndexSet<usize>,
    pub match_evidence: Box<crate::MatchEvidence>,
}

impl Valid {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn merge_from(&mut self, other: Self) {
        self.properties.extend(other.properties);
        self.indices.extend(other.indices);
        self.match_evidence.merge_from(*other.match_evidence);
    }

    /// Merges the annotations a subschema produced into `self` and returns
    /// whatever failure is left for the caller to report.
    ///
    /// Annotations of a subschema whose assertions failed are discarded,
    /// because a failing subschema contributes nothing to
    /// `unevaluatedProperties` / `unevaluatedItems`.
    pub fn merge_result(&mut self, result: Result<Self, crate::Invalid>) -> Option<crate::Invalid> {
        match result {
            Ok(local_evaluated_locations) => {
                self.merge_from(local_evaluated_locations);
                None
            }
            Err(mut error) => {
                if !error.assertion_failed {
                    self.merge_from(std::mem::take(&mut error.local_evaluated_locations));
                }
                Some(error)
            }
        }
    }

    #[inline]
    pub fn mark_property(&mut self, key: impl Into<String>) {
        self.properties.insert(key.into());
    }

    #[inline]
    pub fn mark_index(&mut self, index: usize) {
        self.indices.insert(index);
    }
}
