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

    #[inline]
    pub fn mark_property(&mut self, key: impl Into<String>) {
        self.properties.insert(key.into());
    }

    #[inline]
    pub fn mark_index(&mut self, index: usize) {
        self.indices.insert(index);
    }
}
