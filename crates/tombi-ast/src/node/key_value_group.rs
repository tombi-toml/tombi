use crate::{DanglingComment, DanglingComments, KeyValue};

/// A group of consecutive KeyValue nodes or dangling comments separated by empty lines.
///
/// This is a logical grouping concept rather than a concrete syntax node type.
/// KeyValueGroup represents either:
/// - A sequence of key-value pairs with their associated dangling comments
/// - A standalone dangling comments group (empty element group)
#[derive(Debug, Clone)]
pub enum KeyValueGroup {
    /// Key-values in the group.
    KeyValues(Vec<KeyValue>),
    /// Dangling comments without any key-values (empty element group)
    DanglingComments(DanglingComments),
}

impl KeyValueGroup {
    /// Create a new KeyValueGroup from a vector of KeyValue nodes.
    pub fn new_key_values(key_values: Vec<KeyValue>) -> Self {
        Self::KeyValues(key_values)
    }

    /// Create a new KeyValueGroup from dangling comments only.
    pub fn new_dangling_comments(comments: DanglingComments) -> Self {
        Self::DanglingComments(comments)
    }

    /// Returns true if this is an empty element group (no key-values)
    pub fn is_empty(&self) -> bool {
        match self {
            Self::KeyValues(key_values) => key_values.is_empty(),
            Self::DanglingComments(_) => true,
        }
    }

    /// Returns the number of KeyValue nodes in this group.
    pub fn len(&self) -> usize {
        match self {
            Self::KeyValues(key_values) => key_values.len(),
            Self::DanglingComments(_) => 0,
        }
    }

    /// Returns an iterator over the KeyValue nodes in this group.
    pub fn key_values(&self) -> impl Iterator<Item = KeyValue> + '_ {
        match self {
            Self::KeyValues(key_values) => itertools::Either::Left(key_values.iter().cloned()),
            Self::DanglingComments(_) => itertools::Either::Right(std::iter::empty()),
        }
    }

    /// Returns dangling comments for empty element groups.
    ///
    /// This is intended for groups where `is_empty() == true`.
    /// For non-empty groups, use `begin_dangling_comments()` / `end_dangling_comments()`.
    pub fn dangling_comments(&self) -> Vec<DanglingComment> {
        debug_assert!(
            self.is_empty(),
            "dangling_comments() is for empty groups; use begin_dangling_comments() / end_dangling_comments() for non-empty groups"
        );

        match self {
            Self::KeyValues(_) => Vec::with_capacity(0),
            Self::DanglingComments(dangling) => {
                dangling.comments().map(DanglingComment::from).collect()
            }
        }
    }
}
