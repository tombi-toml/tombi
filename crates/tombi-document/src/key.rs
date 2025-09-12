use serde::forward_to_deserialize_any;
use tombi_toml_version::TomlVersion;

use crate::IntoDocument;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyKind {
    BareKey,
    BasicString,
    LiteralString,
}

impl From<tombi_document_tree::KeyKind> for KeyKind {
    fn from(kind: tombi_document_tree::KeyKind) -> Self {
        match kind {
            tombi_document_tree::KeyKind::BareKey => Self::BareKey,
            tombi_document_tree::KeyKind::BasicString => Self::BasicString,
            tombi_document_tree::KeyKind::LiteralString => Self::LiteralString,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Key {
    kind: KeyKind,
    value: String,
}

impl Key {
    pub fn new(kind: KeyKind, value: String) -> Self {
        Self { kind, value }
    }

    #[inline]
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Key {}

impl From<crate::String> for Key {
    fn from(value: crate::String) -> Self {
        Self {
            kind: match value.kind() {
                _ if !value.value.contains("'") && !value.value.contains("\"") => KeyKind::BareKey,
                crate::StringKind::BasicString => KeyKind::BasicString,
                crate::StringKind::LiteralString => KeyKind::LiteralString,
                crate::StringKind::MultiLineBasicString => KeyKind::BasicString,
                crate::StringKind::MultiLineLiteralString => KeyKind::LiteralString,
            },
            value: value.value,
        }
    }
}

impl std::hash::Hash for Key {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state)
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            KeyKind::BareKey => write!(f, "{}", self.value),
            KeyKind::BasicString => write!(f, r#""{}""#, self.value),
            KeyKind::LiteralString => write!(f, r#"'{}'"#, self.value),
        }
    }
}

impl IntoDocument<Key> for tombi_document_tree::Key {
    fn into_document(self, _toml_version: TomlVersion) -> Key {
        Key {
            kind: self.kind().into(),
            value: self.value,
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if !value.contains("'") && !value.contains("\"") {
            Ok(Self {
                kind: KeyKind::BareKey,
                value,
            })
        } else if value.contains('"') && !value.contains('\'') {
            Ok(Self {
                kind: KeyKind::LiteralString,
                value: format!("'{value}'"),
            })
        } else {
            Ok(Self {
                kind: KeyKind::BasicString,
                value: format!(r#""{}""#, value.replace("\"", r#"\""#)),
            })
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::de::Deserializer<'de> for &'de Key {
    type Error = crate::de::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_str(self.value())
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier enum ignored_any
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::test_deserialize;

    test_deserialize! {
        #[test]
        fn bare_key(r#"key = 1"#) -> Ok(json!({"key": 1}))
    }

    test_deserialize! {
        #[test]
        fn basic_string_key(r#""key" = 1"#) -> Ok(json!({"key": 1}))
    }

    test_deserialize! {
        #[test]
        fn literal_string_key(r#"'key' = 'value'"#) -> Ok(json!({"key": "value"}))
    }

    test_deserialize! {
        #[test]
        fn boolean_key(r#"true = 'value'"#) -> Ok(json!({"true": "value"}))
    }

    test_deserialize! {
        #[test]
        fn integer_key(r#"123 = 'value'"#) -> Ok(json!({"123":  "value"}))
    }

    test_deserialize! {
        #[test]
        fn float_key1(r#"3.14 = 'value'"#) -> Ok(json!({"3": {"14": "value"}}))
    }

    test_deserialize! {
        #[test]
        fn nan_float_key(r#"nan = 'value'"#) -> Ok(json!({"nan": "value"}))
    }

    test_deserialize! {
        #[test]
        fn p_nan_float_key(r#"+nan = 'value'"#) -> Err([
            ("invalid string: bare key contains '+' character", ((0, 0), (0, 4)))
        ])
    }

    test_deserialize! {
        #[test]
        fn m_nan_float_key(r#"-nan = 'value'"#) -> Ok(json!({"-nan": "value"}))
    }

    test_deserialize! {
        #[test]
        fn float_and_bare_key(r#"3.14.abc = 'value'"#) -> Ok(json!({"3": {"14": {"abc": "value"}}}))
    }
}
