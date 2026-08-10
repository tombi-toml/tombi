pub type InstancePath = Vec<tombi_schema_store::Accessor>;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MatchEvidence {
    root_value_assertions: u32,
    matched_root_value_assertions: u32,
    root_singleton_assertions: u32,
    matched_root_singleton_assertions: u32,
    type_assertions: u32,
    matched_type_assertions: u32,
    primary_value_locations: tombi_hashmap::IndexSet<InstancePath>,
    declared_child_value_locations: tombi_hashmap::IndexSet<InstancePath>,
    declared_child_locations: tombi_hashmap::IndexSet<InstancePath>,
    required_locations: tombi_hashmap::IndexSet<InstancePath>,
    fallback_child_value_locations: tombi_hashmap::IndexSet<InstancePath>,
}

impl MatchEvidence {
    #[inline]
    pub fn merge_from(&mut self, other: Self) {
        self.root_value_assertions += other.root_value_assertions;
        self.matched_root_value_assertions += other.matched_root_value_assertions;
        self.root_singleton_assertions += other.root_singleton_assertions;
        self.matched_root_singleton_assertions += other.matched_root_singleton_assertions;
        self.type_assertions += other.type_assertions;
        self.matched_type_assertions += other.matched_type_assertions;
        self.primary_value_locations
            .extend(other.primary_value_locations);
        self.declared_child_value_locations
            .extend(other.declared_child_value_locations);
        self.declared_child_locations
            .extend(other.declared_child_locations);
        self.required_locations.extend(other.required_locations);
        self.fallback_child_value_locations
            .extend(other.fallback_child_value_locations);
    }

    /// Merge evidence produced below the current instance without treating a
    /// child's root assertions as assertions on the current instance.
    #[inline]
    pub fn merge_descendant_from(&mut self, other: &Self) {
        self.primary_value_locations
            .extend(other.primary_value_locations.iter().cloned());
        self.declared_child_value_locations
            .extend(other.declared_child_value_locations.iter().cloned());
        self.declared_child_locations
            .extend(other.declared_child_locations.iter().cloned());
        self.required_locations
            .extend(other.required_locations.iter().cloned());
        self.fallback_child_value_locations
            .extend(other.fallback_child_value_locations.iter().cloned());
    }

    #[inline]
    pub fn mark_root_value_assertion(&mut self, matched: bool, singleton: bool) {
        self.root_value_assertions += 1;
        self.matched_root_value_assertions += u32::from(matched);
        if singleton {
            self.root_singleton_assertions += 1;
            self.matched_root_singleton_assertions += u32::from(matched);
        }
    }

    #[inline]
    pub fn mark_type_assertion(&mut self, matched: bool) {
        self.type_assertions += 1;
        self.matched_type_assertions += u32::from(matched);
    }

    #[inline]
    pub fn root_singleton_matched(&self) -> bool {
        self.root_singleton_assertions > 0
            && self.root_singleton_assertions == self.matched_root_singleton_assertions
    }

    #[inline]
    pub fn mark_primary_value(&mut self, path: InstancePath) {
        self.primary_value_locations.insert(path);
    }

    #[inline]
    pub fn mark_declared_child(&mut self, path: InstancePath, value_matched: bool) {
        self.declared_child_locations.insert(path.clone());
        if value_matched {
            self.declared_child_value_locations.insert(path);
        }
    }

    #[inline]
    pub fn mark_required(&mut self, path: InstancePath) {
        self.required_locations.insert(path);
    }

    #[inline]
    pub fn mark_fallback_child_value(&mut self, path: InstancePath) {
        self.fallback_child_value_locations.insert(path);
    }

    #[inline]
    pub fn score(&self) -> MatchScore {
        MatchScore {
            root_value_matched: self.root_value_assertions > 0
                && self.root_value_assertions == self.matched_root_value_assertions,
            primary_value_matches: self.primary_value_locations.len(),
            type_assertion_matched: self.type_assertions > 0
                && self.type_assertions == self.matched_type_assertions,
            declared_child_value_matches: self.declared_child_value_locations.len(),
            declared_child_matches: self.declared_child_locations.len(),
            required_matches: self.required_locations.len(),
            fallback_child_value_matches: self.fallback_child_value_locations.len(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MatchScore {
    pub root_value_matched: bool,
    pub primary_value_matches: usize,
    pub type_assertion_matched: bool,
    pub declared_child_value_matches: usize,
    pub declared_child_matches: usize,
    pub required_matches: usize,
    pub fallback_child_value_matches: usize,
}

#[cfg(test)]
mod tests {
    use super::MatchEvidence;
    use tombi_schema_store::Accessor;

    fn key_path(key: &str) -> Vec<Accessor> {
        vec![Accessor::Key(key.to_string())]
    }

    #[test]
    fn location_evidence_is_deduplicated_by_absolute_path() {
        let mut evidence = MatchEvidence::default();
        evidence.mark_declared_child(key_path("kind"), true);
        evidence.mark_declared_child(key_path("kind"), true);
        evidence.mark_required(key_path("kind"));
        evidence.mark_required(key_path("kind"));

        let score = evidence.score();
        assert_eq!(score.declared_child_value_matches, 1);
        assert_eq!(score.declared_child_matches, 1);
        assert_eq!(score.required_matches, 1);
    }

    #[test]
    fn all_root_assertions_must_match() {
        let mut evidence = MatchEvidence::default();
        evidence.mark_root_value_assertion(true, true);
        evidence.mark_root_value_assertion(false, true);

        assert!(!evidence.root_singleton_matched());
        assert!(!evidence.score().root_value_matched);
    }

    #[test]
    fn descendant_merge_does_not_promote_child_root_assertions() {
        let mut child = MatchEvidence::default();
        child.mark_root_value_assertion(true, true);
        child.mark_declared_child(key_path("nested"), true);

        let mut parent = MatchEvidence::default();
        parent.merge_descendant_from(&child);

        assert!(!parent.root_singleton_matched());
        assert_eq!(parent.score().declared_child_value_matches, 1);
    }
}
