use crate::{DanglingComment, DanglingComments, Value};

/// A group of consecutive Value nodes or dangling comments separated by empty lines.
///
/// This is a logical grouping concept rather than a concrete syntax node type.
/// ValueGroup represents either:
/// - A sequence of values with their associated dangling comments
/// - A standalone dangling comments group (empty element group)
#[derive(Debug, Clone)]
pub enum ValueGroup {
    /// Values in the group.
    Values(Vec<Value>),
    /// Dangling comments without any values (empty element group)
    DanglingComments(DanglingComments),
}

impl ValueGroup {
    /// Create a new ValueGroup from a vector of Value nodes.
    pub fn new(values: Vec<Value>) -> Self {
        Self::Values(values)
    }

    /// Create a new ValueGroup from dangling comments only.
    pub fn from_dangling_comments(comments: DanglingComments) -> Self {
        Self::DanglingComments(comments)
    }

    /// Returns true if this is an empty element group (no values)
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Values(values) => values.is_empty(),
            Self::DanglingComments(_) => true,
        }
    }

    /// Returns the number of Value nodes in this group.
    pub fn len(&self) -> usize {
        match self {
            Self::Values(values) => values.len(),
            Self::DanglingComments(_) => 0,
        }
    }

    /// Returns an iterator over the Value nodes in this group.
    pub fn values(&self) -> impl Iterator<Item = Value> + '_ {
        match self {
            Self::Values(values) => itertools::Either::Left(values.iter().cloned()),
            Self::DanglingComments(_) => itertools::Either::Right(std::iter::empty()),
        }
    }

    /// Returns dangling comments for empty element groups.
    ///
    /// This is intended for groups where `is_empty() == true`.
    /// For non-empty groups, use `begin_dangling_comments()` / `end_dangling_comments()`.
    pub fn dangling_comments(&self) -> Vec<DanglingComment> {
        match self {
            Self::Values(_) => Vec::with_capacity(0),
            Self::DanglingComments(dangling) => {
                dangling.comments().map(DanglingComment::from).collect()
            }
        }
    }
}
