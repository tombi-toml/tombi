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

    /// Merges the annotations a validation result carries into `self` and
    /// returns whatever failure is left for the caller to report.
    ///
    /// The caller is responsible for having dropped the annotations of any
    /// subschema whose own assertions failed. Use this when the result is the
    /// merged output of several subschemas that already applied that rule
    /// individually — as `validate_if_then_else` does — so that a failing
    /// branch does not take its successful siblings' annotations with it.
    pub(crate) fn merge_result_keeping_annotations(
        &mut self,
        result: Result<Self, crate::Invalid>,
    ) -> Option<crate::Invalid> {
        match result {
            Ok(local_evaluated_locations) => {
                self.merge_from(local_evaluated_locations);
                None
            }
            Err(mut error) => {
                self.merge_from(std::mem::take(&mut error.local_evaluated_locations));
                Some(error)
            }
        }
    }

    /// Merges the annotations a single subschema produced into `self`, dropping
    /// them when that subschema failed its own assertions, and returns whatever
    /// failure is left for the caller to report.
    pub(crate) fn merge_result(
        &mut self,
        mut result: Result<Self, crate::Invalid>,
    ) -> Option<crate::Invalid> {
        crate::validate::discard_failed_annotations(&mut result);

        self.merge_result_keeping_annotations(result)
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
