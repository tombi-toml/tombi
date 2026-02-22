pub mod de;
mod key;
mod value;

use std::ops::{Deref, DerefMut};

pub use key::{Key, KeyKind};
use serde::forward_to_deserialize_any;
use tombi_toml_version::TomlVersion;
pub use value::{
    Array, ArrayKind, Boolean, Float, Integer, IntegerKind, LocalDate, LocalDateTime, LocalTime,
    OffsetDateTime, String, StringKind, Table, TableKind, TimeZoneOffset, Value, ValueKind,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Document(pub(crate) Table);

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Document {
    pub fn new() -> Self {
        Self(Table::new(TableKind::Table))
    }
}

impl From<Document> for Table {
    fn from(document: Document) -> Self {
        document.0
    }
}

impl From<Table> for Document {
    fn from(table: Table) -> Self {
        Self(table)
    }
}

impl Deref for Document {
    type Target = Table;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Document {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub trait IntoDocument<T> {
    fn into_document(self, toml_version: TomlVersion) -> T;
}

impl IntoDocument<Document> for tombi_document_tree::DocumentTree {
    fn into_document(self, toml_version: TomlVersion) -> Document {
        Document(tombi_document_tree::Table::from(self).into_document(toml_version))
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Document {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Document {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let table = Table::deserialize(deserializer)?;
        Ok(Document(table))
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserializer<'de> for &'de Document {
    type Error = crate::de::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.0.deserialize_any(visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct struct map identifier enum ignored_any
    }
}

#[cfg(test)]
#[macro_export]
macro_rules! test_deserialize {
    {#[test] fn $name:ident($source:expr) -> Ok($json:expr)} => {
        test_deserialize! {#[test] fn $name($source, tombi_toml_version::TomlVersion::default()) -> Ok($json)}
    };

    {#[test] fn $name:ident($source:expr, $toml_version:expr) -> Ok($json:expr)} => {
        #[cfg(feature = "serde")]
        #[test]
        fn $name() {
            use tombi_ast::AstNode;
            use tombi_document_tree::IntoDocumentTreeAndErrors;
            use $crate::IntoDocument;

            tombi_test_lib::init_log();

            let source = textwrap::dedent($source);
            let p = tombi_parser::parse(&source.trim());
            pretty_assertions::assert_eq!(p.errors, Vec::<tombi_parser::Error>::new());
            let root = tombi_ast::Root::cast(p.into_syntax_node()).unwrap();
            let (document_tree, errors) = root.into_document_tree_and_errors($toml_version).into();
            pretty_assertions::assert_eq!(errors, vec![]);
            let document: $crate::Document = document_tree.into_document($toml_version);
            let serialized = serde_json::to_string(&document).unwrap();
            pretty_assertions::assert_eq!(serialized, $json.to_string());
        }
    };

    {#[test] fn $name:ident($source:expr) -> Err($errors:expr)} => {
        test_deserialize! {#[test] fn $name($source, tombi_toml_version::TomlVersion::default()) -> Err($errors)}
    };

    {#[test] fn $name:ident($source:expr, $toml_version:expr) -> Err($errors:expr)} => {
        #[cfg(feature = "serde")]
        #[test]
        fn $name() {
            use tombi_ast::AstNode;
            use itertools::Itertools;
            use tombi_document_tree::IntoDocumentTreeAndErrors;

            let source = textwrap::dedent($source);
            let p = tombi_parser::parse(&source.trim());
            let expected_errors = $errors
                .into_iter()
                .map(|(m, r)| (m.to_string(), tombi_text::Range::from(r)))
                .collect_vec();

            if !p.errors.is_empty() {
                pretty_assertions::assert_eq!(
                    p.errors
                        .iter()
                        .map(|e| (e.to_message(), e.range()))
                        .collect_vec(),
                    expected_errors,
                );
            }
            let root = tombi_ast::Root::cast(p.into_syntax_node()).unwrap();
            let (_, errs) = root.into_document_tree_and_errors($toml_version).into();
            pretty_assertions::assert_eq!(
                errs
                    .iter()
                    .map(|e| (e.to_message(), e.range()))
                    .collect_vec(),
                expected_errors
            );
        }
    };
}

#[cfg(test)]
mod test {
    use serde_json::json;

    test_deserialize! {
        #[test]
        fn empty("") -> Ok(json!({}))
    }

    test_deserialize! {
        #[test]
        fn key_values(
            r#"
            key = "value"
            flag = true
            "#
        ) -> Ok(json!({"key": "value", "flag": true}))
    }

    test_deserialize! {
        #[test]
        fn array_of_tables_sample(
            r#"
            [[fruits]]
            name = "apple"

            [fruits.physical]  # subtable
            color = "red"
            shape = "round"

            [[fruits.varieties]]  # nested array of tables
            name = "red delicious"

            [[fruits.varieties]]
            name = "granny smith"

            [[fruits]]
            name = "banana"

            [[fruits.varieties]]
            name = "plantain"
            "#
        ) -> Ok(
            json!(
                {
                    "fruits": [
                        {
                            "name": "apple",
                            "physical": {
                                "color": "red",
                                "shape": "round"
                            },
                            "varieties": [
                                { "name": "red delicious" },
                                { "name": "granny smith" }
                            ]
                        },
                        {
                            "name": "banana",
                            "varieties": [
                                { "name": "plantain" }
                            ]
                        }
                    ]
                }
            )
        )
    }
}
