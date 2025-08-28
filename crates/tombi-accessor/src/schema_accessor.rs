use itertools::Itertools;

use crate::Accessor;

/// Represents an accessor to a value in a TOML-like structure.
/// It can either be a key (for objects) or an index (for arrays).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SchemaAccessor {
    Key(String),
    Index,
}

impl SchemaAccessor {
    /// Parse a schema access path into a sequence of accessors.
    ///
    /// # Examples
    ///
    /// ```
    /// use tombi_schema_store::{SchemaAccessor, Accessor};
    ///
    /// let accessors = SchemaAccessor::parse("key1[*].key2").unwrap();
    /// assert_eq!(accessors.len(), 3);
    /// assert_eq!(accessors[0], SchemaAccessor::Key("key1".to_string()));
    /// assert_eq!(accessors[1], SchemaAccessor::Index);
    /// assert_eq!(accessors[2], SchemaAccessor::Key("key2".to_string()));
    /// ```
    pub fn parse(path: &str) -> Option<Vec<SchemaAccessor>> {
        let mut accessors = Vec::new();
        let mut current_key = String::new();

        if path.is_empty() {
            return None;
        }

        let chars: Vec<char> = path.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            match chars[i] {
                '[' => {
                    if !current_key.is_empty() {
                        accessors.push(SchemaAccessor::Key(current_key));
                        current_key = String::new();
                    }
                    i += 1;
                    let mut index_str = String::new();
                    while i < chars.len() && chars[i] != ']' {
                        index_str.push(chars[i]);
                        i += 1;
                    }
                    if index_str == "*" {
                        accessors.push(SchemaAccessor::Index); // Use 0 as a placeholder for [*]
                    } else if index_str.parse::<usize>().is_ok() {
                        accessors.push(SchemaAccessor::Index);
                    } else {
                        tracing::warn!("Invalid schema accessor: {path}");
                        return None;
                    }
                }
                '.' => {
                    if !current_key.is_empty() {
                        accessors.push(SchemaAccessor::Key(current_key));
                        current_key = String::new();
                    }
                }
                c => {
                    current_key.push(c);
                }
            }
            i += 1;
        }

        if !current_key.is_empty() {
            accessors.push(SchemaAccessor::Key(current_key));
        }

        Some(accessors)
    }
}

impl PartialEq<Accessor> for SchemaAccessor {
    fn eq(&self, other: &Accessor) -> bool {
        match (self, other) {
            (SchemaAccessor::Key(key1), Accessor::Key(key2)) => key1 == key2,
            (SchemaAccessor::Index, Accessor::Index(_)) => true,
            _ => false,
        }
    }
}

impl PartialOrd<SchemaAccessor> for SchemaAccessor {
    fn partial_cmp(&self, other: &SchemaAccessor) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (SchemaAccessor::Key(key1), SchemaAccessor::Key(key2)) => key1.partial_cmp(key2),
            (SchemaAccessor::Index, _) | (_, SchemaAccessor::Index) => None,
        }
    }
}

impl From<Accessor> for SchemaAccessor {
    fn from(accessor: Accessor) -> Self {
        match accessor {
            Accessor::Key(key) => SchemaAccessor::Key(key),
            Accessor::Index(_) => SchemaAccessor::Index,
        }
    }
}

impl From<&Accessor> for SchemaAccessor {
    fn from(value: &Accessor) -> Self {
        match value {
            Accessor::Key(key) => SchemaAccessor::Key(key.clone()),
            Accessor::Index(_) => SchemaAccessor::Index,
        }
    }
}

impl std::fmt::Display for SchemaAccessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaAccessor::Key(key) => write!(f, "{key}"),
            SchemaAccessor::Index => write!(f, "[*]"),
        }
    }
}

/// A collection of `Accessor`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct SchemaAccessors(Vec<SchemaAccessor>);

impl SchemaAccessors {
    #[inline]
    pub fn first(&self) -> Option<&SchemaAccessor> {
        self.0.first()
    }

    #[inline]
    pub fn last(&self) -> Option<&SchemaAccessor> {
        self.0.last()
    }
}

impl AsRef<[SchemaAccessor]> for SchemaAccessors {
    fn as_ref(&self) -> &[SchemaAccessor] {
        &self.0
    }
}

impl From<&[Accessor]> for SchemaAccessors {
    fn from(accessors: &[Accessor]) -> Self {
        Self(accessors.into_iter().map(Into::into).collect_vec())
    }
}

impl From<&Vec<Accessor>> for SchemaAccessors {
    fn from(accessors: &Vec<Accessor>) -> Self {
        Self(accessors.into_iter().map(Into::into).collect_vec())
    }
}

impl From<&[SchemaAccessor]> for SchemaAccessors {
    fn from(accessors: &[SchemaAccessor]) -> Self {
        Self(accessors.to_vec())
    }
}

impl From<Vec<SchemaAccessor>> for SchemaAccessors {
    fn from(accessors: Vec<SchemaAccessor>) -> Self {
        Self(accessors)
    }
}

impl std::fmt::Display for SchemaAccessors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.0.iter();
        if let Some(accessor) = iter.next() {
            write!(f, "{accessor}")?;
            for accessor in iter {
                match accessor {
                    SchemaAccessor::Key(_) => write!(f, ".{accessor}")?,
                    SchemaAccessor::Index => write!(f, "{accessor}")?,
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("key1[*].key2", vec![
        SchemaAccessor::Key("key1".to_string()),
        SchemaAccessor::Index,
        SchemaAccessor::Key("key2".to_string()),
    ])]
    #[case("key1[0].key2", vec![
        SchemaAccessor::Key("key1".to_string()),
        SchemaAccessor::Index,
        SchemaAccessor::Key("key2".to_string()),
    ])]
    #[case("simple.key", vec![
        SchemaAccessor::Key("simple".to_string()),
        SchemaAccessor::Key("key".to_string()),
    ])]
    #[case("array[5]", vec![
        SchemaAccessor::Key("array".to_string()),
        SchemaAccessor::Index,
    ])]
    fn test_schema_accessor(#[case] input: &str, #[case] expected: Vec<SchemaAccessor>) {
        let result = SchemaAccessor::parse(input).unwrap();
        pretty_assertions::assert_eq!(result, expected, "Failed for input: {}", input);
    }
}
