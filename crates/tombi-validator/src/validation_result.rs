use std::collections::BTreeSet;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EvaluatedLocations {
    pub properties: BTreeSet<String>,
    pub indices: BTreeSet<usize>,
}

impl EvaluatedLocations {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn merge_from(&mut self, other: Self) {
        self.properties.extend(other.properties);
        self.indices.extend(other.indices);
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
