use itertools::Itertools;
use tombi_toml_version::TomlVersion;

use crate::Accessor;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PatternAccessor {
    Key(String),
    AnyKey,
    Index,
}

impl PatternAccessor {
    pub fn parse(path: &str) -> Option<Vec<PatternAccessor>> {
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
                        accessors.push(parse_key_or_wildcard(current_key));
                        current_key = String::new();
                    }
                    i += 1;
                    let mut index_str = String::new();
                    while i < chars.len() && chars[i] != ']' {
                        index_str.push(chars[i]);
                        i += 1;
                    }
                    if i >= chars.len() || chars[i] != ']' {
                        return None;
                    }
                    if index_str == "*" || index_str.parse::<usize>().is_ok() {
                        accessors.push(PatternAccessor::Index);
                    } else {
                        return None;
                    }
                }
                '\'' | '"' if current_key.is_empty() => {
                    let (quoted_key, next_index) = parse_quoted_key(&chars, i)?;
                    accessors.push(PatternAccessor::Key(quoted_key));
                    i = next_index;
                }
                '.' => {
                    if !current_key.is_empty() {
                        accessors.push(parse_key_or_wildcard(current_key));
                        current_key = String::new();
                    }
                }
                c => current_key.push(c),
            }
            i += 1;
        }

        if !current_key.is_empty() {
            accessors.push(parse_key_or_wildcard(current_key));
        }

        Some(accessors)
    }
}

impl PartialEq<Accessor> for PatternAccessor {
    fn eq(&self, other: &Accessor) -> bool {
        match (self, other) {
            (PatternAccessor::Key(expected), Accessor::Key(actual)) => expected == actual,
            (PatternAccessor::AnyKey, Accessor::Key(_)) => true,
            (PatternAccessor::Index, Accessor::Index(_)) => true,
            _ => false,
        }
    }
}

fn parse_key_or_wildcard(key: String) -> PatternAccessor {
    if key == "*" {
        PatternAccessor::AnyKey
    } else {
        PatternAccessor::Key(key)
    }
}

fn parse_quoted_key(chars: &[char], start: usize) -> Option<(String, usize)> {
    let quote = chars[start];
    let mut end = start + 1;
    let mut escaped = false;

    while end < chars.len() {
        let current = chars[end];
        if quote == '"' && current == '\\' && !escaped {
            escaped = true;
            end += 1;
            continue;
        }
        if current == quote && !escaped {
            let quoted = chars[start..=end].iter().collect::<String>();
            let parsed = match quote {
                '"' => tombi_toml_text::try_from_basic_string(&quoted, TomlVersion::V1_0_0).ok()?,
                '\'' => tombi_toml_text::try_from_literal_string(&quoted).ok()?,
                _ => return None,
            };
            return Some((parsed, end));
        }
        escaped = false;
        end += 1;
    }

    None
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct PatternAccessors(Vec<PatternAccessor>);

impl From<Vec<PatternAccessor>> for PatternAccessors {
    fn from(accessors: Vec<PatternAccessor>) -> Self {
        Self(accessors)
    }
}

impl From<&[PatternAccessor]> for PatternAccessors {
    fn from(accessors: &[PatternAccessor]) -> Self {
        Self(accessors.to_vec())
    }
}

impl std::fmt::Display for PatternAccessors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.0.iter();
        if let Some(accessor) = iter.next() {
            write!(f, "{accessor}")?;
            for accessor in iter {
                match accessor {
                    PatternAccessor::Key(_) | PatternAccessor::AnyKey => write!(f, ".{accessor}")?,
                    PatternAccessor::Index => write!(f, "{accessor}")?,
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for PatternAccessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatternAccessor::Key(key) => write!(f, "{}", tombi_toml_text::to_key_string(key)),
            PatternAccessor::AnyKey => write!(f, "*"),
            PatternAccessor::Index => write!(f, "[*]"),
        }
    }
}

impl From<&[Accessor]> for PatternAccessors {
    fn from(accessors: &[Accessor]) -> Self {
        Self(
            accessors
                .iter()
                .map(|accessor| match accessor {
                    Accessor::Key(key) => PatternAccessor::Key(key.clone()),
                    Accessor::Index(_) => PatternAccessor::Index,
                })
                .collect_vec(),
        )
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("tool.*", vec![
        PatternAccessor::Key("tool".to_string()),
        PatternAccessor::AnyKey,
    ])]
    #[case("tool.'*'", vec![
        PatternAccessor::Key("tool".to_string()),
        PatternAccessor::Key("*".to_string()),
    ])]
    #[case("tool.\"*\"", vec![
        PatternAccessor::Key("tool".to_string()),
        PatternAccessor::Key("*".to_string()),
    ])]
    #[case("\"a.b\".c", vec![
        PatternAccessor::Key("a.b".to_string()),
        PatternAccessor::Key("c".to_string()),
    ])]
    #[case("items[*].name", vec![
        PatternAccessor::Key("items".to_string()),
        PatternAccessor::Index,
        PatternAccessor::Key("name".to_string()),
    ])]
    fn test_root_accessor_parse(#[case] input: &str, #[case] expected: Vec<PatternAccessor>) {
        let result = PatternAccessor::parse(input).unwrap();
        pretty_assertions::assert_eq!(result, expected, "Failed for input: {}", input);
    }

    #[test]
    fn test_root_accessors_display() {
        let accessors = PatternAccessors::from(vec![
            PatternAccessor::Key("tool".to_string()),
            PatternAccessor::AnyKey,
            PatternAccessor::Key("enabled".to_string()),
        ]);
        assert_eq!(format!("{accessors}"), "tool.*.enabled");
    }

    #[test]
    fn test_root_accessors_display_literal_asterisk_key() {
        let accessors = PatternAccessors::from(vec![
            PatternAccessor::Key("tool".to_string()),
            PatternAccessor::Key("*".to_string()),
        ]);
        assert_eq!(format!("{accessors}"), r#"tool."*""#);
    }

    #[rstest]
    #[case("")]
    #[case("items[*")]
    #[case("items[foo]")]
    fn test_root_accessor_parse_invalid(#[case] input: &str) {
        assert_eq!(PatternAccessor::parse(input), None);
    }

    #[test]
    fn test_root_accessor_matches() {
        assert_eq!(
            PatternAccessor::AnyKey,
            Accessor::Key("taskipy".to_string())
        );
        assert_eq!(PatternAccessor::Index, Accessor::Index(3));
        assert_eq!(
            PatternAccessor::Key("tool".to_string()),
            Accessor::Key("tool".to_string())
        );
        assert_ne!(
            PatternAccessor::Key("tool".to_string()),
            Accessor::Key("other".to_string())
        );
    }
}
