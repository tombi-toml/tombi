use itertools::Itertools;
use tombi_toml_version::TomlVersion;

use crate::Accessor;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RootAccessor {
    Key(String),
    AnyKey,
    Index,
}

impl RootAccessor {
    pub fn parse(path: &str) -> Option<Vec<RootAccessor>> {
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
                    if index_str == "*" || index_str.parse::<usize>().is_ok() {
                        accessors.push(RootAccessor::Index);
                    } else {
                        return None;
                    }
                }
                '\'' | '"' if current_key.is_empty() => {
                    let (quoted_key, next_index) = parse_quoted_key(&chars, i)?;
                    accessors.push(RootAccessor::Key(quoted_key));
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

impl PartialEq<Accessor> for RootAccessor {
    fn eq(&self, other: &Accessor) -> bool {
        match (self, other) {
            (RootAccessor::Key(expected), Accessor::Key(actual)) => expected == actual,
            (RootAccessor::AnyKey, Accessor::Key(_)) => true,
            (RootAccessor::Index, Accessor::Index(_)) => true,
            _ => false,
        }
    }
}

fn parse_key_or_wildcard(key: String) -> RootAccessor {
    if key == "*" {
        RootAccessor::AnyKey
    } else {
        RootAccessor::Key(key)
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
pub struct RootAccessors(Vec<RootAccessor>);

impl From<Vec<RootAccessor>> for RootAccessors {
    fn from(accessors: Vec<RootAccessor>) -> Self {
        Self(accessors)
    }
}

impl From<&[RootAccessor]> for RootAccessors {
    fn from(accessors: &[RootAccessor]) -> Self {
        Self(accessors.to_vec())
    }
}

impl std::fmt::Display for RootAccessors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.0.iter();
        if let Some(accessor) = iter.next() {
            write!(f, "{accessor}")?;
            for accessor in iter {
                match accessor {
                    RootAccessor::Key(_) | RootAccessor::AnyKey => write!(f, ".{accessor}")?,
                    RootAccessor::Index => write!(f, "{accessor}")?,
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for RootAccessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RootAccessor::Key(key) => write!(f, "{}", tombi_toml_text::to_key_string(key)),
            RootAccessor::AnyKey => write!(f, "*"),
            RootAccessor::Index => write!(f, "[*]"),
        }
    }
}

impl From<&[Accessor]> for RootAccessors {
    fn from(accessors: &[Accessor]) -> Self {
        Self(
            accessors
                .iter()
                .map(|accessor| match accessor {
                    Accessor::Key(key) => RootAccessor::Key(key.clone()),
                    Accessor::Index(_) => RootAccessor::Index,
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
        RootAccessor::Key("tool".to_string()),
        RootAccessor::AnyKey,
    ])]
    #[case("tool.'*'", vec![
        RootAccessor::Key("tool".to_string()),
        RootAccessor::Key("*".to_string()),
    ])]
    #[case("tool.\"*\"", vec![
        RootAccessor::Key("tool".to_string()),
        RootAccessor::Key("*".to_string()),
    ])]
    #[case("\"a.b\".c", vec![
        RootAccessor::Key("a.b".to_string()),
        RootAccessor::Key("c".to_string()),
    ])]
    #[case("items[*].name", vec![
        RootAccessor::Key("items".to_string()),
        RootAccessor::Index,
        RootAccessor::Key("name".to_string()),
    ])]
    fn test_root_accessor_parse(#[case] input: &str, #[case] expected: Vec<RootAccessor>) {
        let result = RootAccessor::parse(input).unwrap();
        pretty_assertions::assert_eq!(result, expected, "Failed for input: {}", input);
    }

    #[test]
    fn test_root_accessors_display() {
        let accessors = RootAccessors::from(vec![
            RootAccessor::Key("tool".to_string()),
            RootAccessor::AnyKey,
            RootAccessor::Key("enabled".to_string()),
        ]);
        assert_eq!(format!("{accessors}"), "tool.*.enabled");
    }

    #[test]
    fn test_root_accessors_display_literal_asterisk_key() {
        let accessors = RootAccessors::from(vec![
            RootAccessor::Key("tool".to_string()),
            RootAccessor::Key("*".to_string()),
        ]);
        assert_eq!(format!("{accessors}"), r#"tool."*""#);
    }

    #[test]
    fn test_root_accessor_matches() {
        assert_eq!(RootAccessor::AnyKey, Accessor::Key("taskipy".to_string()));
        assert_eq!(RootAccessor::Index, Accessor::Index(3));
        assert_eq!(
            RootAccessor::Key("tool".to_string()),
            Accessor::Key("tool".to_string())
        );
        assert_ne!(
            RootAccessor::Key("tool".to_string()),
            Accessor::Key("other".to_string())
        );
    }
}
