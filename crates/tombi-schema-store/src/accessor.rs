use crate::SchemaAccessor;

/// Represents an accessor to a value in a TOML-like structure.
/// It can either be a key (for objects) or an index (for arrays).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Accessor {
    Key(String),
    Index(usize),
}

impl Accessor {
    #[inline]
    pub fn is_key(&self) -> bool {
        matches!(self, Accessor::Key(_))
    }

    #[inline]
    pub fn as_key(&self) -> Option<&str> {
        match self {
            Accessor::Key(key) => Some(key),
            _ => None,
        }
    }

    #[inline]
    pub fn is_index(&self) -> bool {
        matches!(self, Accessor::Index(_))
    }

    #[inline]
    pub fn as_index(&self) -> Option<usize> {
        match self {
            Accessor::Index(index) => Some(*index),
            _ => None,
        }
    }
}

impl std::fmt::Display for Accessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Accessor::Key(key) => write!(f, "{key}"),
            Accessor::Index(index) => write!(f, "[{index}]"),
        }
    }
}

impl PartialEq<SchemaAccessor> for Accessor {
    fn eq(&self, other: &SchemaAccessor) -> bool {
        match (self, other) {
            (Accessor::Key(key), SchemaAccessor::Key(other_key)) => key == other_key,
            (Accessor::Index(_), SchemaAccessor::Index) => true,
            _ => false,
        }
    }
}

impl PartialEq<&str> for Accessor {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Accessor::Key(key) => key == *other,
            _ => false,
        }
    }
}

/// A collection of `Accessor`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Accessors(Vec<Accessor>);

impl Accessors {
    #[inline]
    pub fn new(accessors: Vec<Accessor>) -> Self {
        Self(accessors)
    }

    #[inline]
    pub fn first(&self) -> Option<&Accessor> {
        self.0.first()
    }

    #[inline]
    pub fn last(&self) -> Option<&Accessor> {
        self.0.last()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl AsRef<[Accessor]> for Accessors {
    fn as_ref(&self) -> &[Accessor] {
        &self.0
    }
}

impl std::fmt::Display for Accessors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.0.iter();
        if let Some(accessor) = iter.next() {
            write!(f, "{accessor}")?;
            for accessor in iter {
                match accessor {
                    Accessor::Key(_) => write!(f, ".{accessor}")?,
                    Accessor::Index(_) => write!(f, "{accessor}")?,
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessorKeyKind {
    Header,
    Dotted,
    KeyValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyContext {
    pub kind: AccessorKeyKind,
    pub range: tombi_text::Range,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessorContext {
    Key(KeyContext),
    Index,
}
