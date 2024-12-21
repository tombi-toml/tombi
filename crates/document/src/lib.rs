mod key;
mod value;

use std::ops::Deref;

pub use key::Key;
pub use value::{
    Array, ArrayKind, Boolean, Float, Integer, IntegerKind, LocalDate, LocalDateTime, LocalTime,
    OffsetDateTime, String, Table, TableKind, Value,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Document(Table);

impl From<Document> for Table {
    fn from(document: Document) -> Self {
        document.0
    }
}

impl From<document_tree::Root> for Document {
    fn from(document: document_tree::Root) -> Self {
        Self(document_tree::Table::from(document).into())
    }
}

impl Deref for Document {
    type Target = Table;

    fn deref(&self) -> &Self::Target {
        &self.0
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

#[cfg(test)]
#[macro_export]
macro_rules! test_serialize {
    {#[test] fn $name:ident($source:expr) -> Ok($json:expr)} => {
        test_serialize! {#[test] fn $name($source, config::TomlVersion::default()) -> Ok($json)}
    };

    {#[test] fn $name:ident($source:expr, $toml_version:expr) -> Ok($json:expr)} => {
        #[cfg(feature = "serde")]
        #[test]
        fn $name() {
            use ast::AstNode;
            use document_tree::TryIntoDocumentTree;

            let source = textwrap::dedent($source);
            let p = parser::parse(&source.trim(), config::TomlVersion::default());
            pretty_assertions::assert_eq!(p.errors(), &[]);
            let root = ast::Root::cast(p.into_syntax_node()).unwrap();
            match root.try_into_document_tree($toml_version) {
                Ok(document_tree) => {
                    let document: crate::Document = document_tree.into();
                    let serialized = serde_json::to_string(&document).unwrap();
                    pretty_assertions::assert_eq!(serialized, $json.to_string());
                }
                Err(errors) => {
                    pretty_assertions::assert_eq!(errors, vec![]);
                }
            }
        }
    };

    {#[test] fn $name:ident($source:expr) -> Err($errors:expr)} => {
        test_serialize! {#[test] fn $name($source, config::TomlVersion::default()) -> Err($errors)}
    };

    {#[test] fn $name:ident($source:expr, $toml_version:expr) -> Err($errors:expr)} => {
        #[cfg(feature = "serde")]
        #[test]
        fn $name() {
            use ast::AstNode;
            use itertools::Itertools;
            use document_tree::TryIntoDocumentTree;

            let source = textwrap::dedent($source);
            let p = parser::parse(&source.trim(), config::TomlVersion::default());
            let errors = $errors
                .into_iter()
                .map(|(m, r)| (m.to_string(), text::Range::from(r)))
                .collect_vec();

            if !p.errors().is_empty() {
                pretty_assertions::assert_eq!(
                    p.errors()
                        .iter()
                        .map(|e| (e.to_message(), e.range()))
                        .collect_vec(),
                    errors,
                );
            }
            let root = ast::Root::cast(p.into_syntax_node()).unwrap();
            match root.try_into_document_tree($toml_version) {
                Ok(_) => {
                    pretty_assertions::assert_eq!("expected error", "but got success");
                }
                Err(errs) => {
                    pretty_assertions::assert_eq!(
                        errs
                            .iter()
                            .map(|e| (e.to_message(), e.range()))
                            .collect_vec(),
                        errors
                    );
                }
            }
        }
    };
}

#[cfg(test)]
mod test {
    use serde_json::json;

    test_serialize! {
        #[test]
        fn empty("") -> Ok(json!({}))
    }

    test_serialize! {
        #[test]
        fn key_values(
            r#"
            key = "value"
            flag = true
            "#
        ) -> Ok(json!({"key": "value", "flag": true}))
    }

    test_serialize! {
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
