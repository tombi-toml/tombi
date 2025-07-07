mod error;

pub use error::Error;
use itertools::Either;
use serde::de::DeserializeOwned;
use tombi_ast::AstNode;
use tombi_document::IntoDocument;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tombi_schema_store::{SchemaStore, SourceSchema};
use tombi_toml_version::TomlVersion;
use typed_builder::TypedBuilder;

/// Deserialize a TOML string into a Rust data structure.
///
/// # Note
///
/// This function is not yet implemented and will return an error.
/// The example below shows the expected usage once implemented.
///
/// # Examples
///
/// ```
/// use serde::Deserialize;
/// use tokio;
///
/// #[derive(Deserialize)]
/// struct Config {
///     ip: String,
///     port: u16,
///     keys: Vec<String>,
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let toml = r#"
///     ip = "127.0.0.1"
///     port = 8080
///     keys = ["key1", "key2"]
///     "#;
///
///     let config: Config = serde_tombi::from_str_async(toml).await.unwrap();
/// }
/// ```
pub async fn from_str_async<T>(toml_text: &str) -> Result<T, crate::de::Error>
where
    T: DeserializeOwned,
{
    Deserializer::new().from_str_async(toml_text).await
}

pub fn from_document<T>(document: tombi_document::Document) -> Result<T, crate::de::Error>
where
    T: DeserializeOwned,
{
    Deserializer::new().from_document(document)
}

// Actual deserializer implementation
#[derive(TypedBuilder)]
pub struct Deserializer<'de> {
    #[builder(default, setter(into, strip_option))]
    config: Option<&'de ::tombi_config::Config>,

    #[builder(default, setter(into, strip_option))]
    config_path: Option<&'de std::path::Path>,

    #[builder(default, setter(into, strip_option))]
    source_path: Option<&'de std::path::Path>,

    #[builder(default, setter(into, strip_option))]
    schema_store: Option<&'de tombi_schema_store::SchemaStore>,
}

impl Default for Deserializer<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Deserializer<'_> {
    pub fn new() -> Self {
        Self {
            config: None,
            config_path: None,
            source_path: None,
            schema_store: None,
        }
    }

    pub async fn from_str_async<T>(&self, toml_text: &str) -> Result<T, crate::de::Error>
    where
        T: DeserializeOwned,
    {
        let toml_version = self.get_toml_version(toml_text).await?;
        let parsed = tombi_parser::parse(toml_text, toml_version);
        let root = tombi_ast::Root::cast(parsed.syntax_node()).expect("AST Root must be present");
        // Check if there are any parsing errors
        if !parsed.errors.is_empty() {
            return Err(parsed.errors.into());
        }
        from_document(self.try_to_document(root, toml_version)?)
    }

    pub fn from_document<T>(
        &self,
        document: tombi_document::Document,
    ) -> Result<T, crate::de::Error>
    where
        T: DeserializeOwned,
    {
        Ok(T::deserialize(&document)?)
    }

    async fn get_toml_version(&self, toml_text: &str) -> Result<TomlVersion, crate::de::Error> {
        let schema_store = match self.schema_store {
            Some(schema_store) => schema_store,
            None => &SchemaStore::new(),
        };

        let mut toml_version = TomlVersion::default();

        if self.schema_store.is_none() {
            match self.config {
                Some(config) => {
                    if let Some(new_toml_version) = config.toml_version {
                        toml_version = new_toml_version;
                    }
                    if self.schema_store.is_none() {
                        schema_store.load_config(config, self.config_path).await?;
                    }
                }
                None => {
                    let (config, config_path) = crate::config::load_with_path()?;

                    if let Some(new_toml_version) = config.toml_version {
                        toml_version = new_toml_version;
                    }

                    schema_store
                        .load_config(&config, config_path.as_deref())
                        .await?;
                }
            }
        }

        let parsed = tombi_parser::parse_document_header_comments(toml_text)
            .cast::<tombi_ast::Root>()
            .expect("AST Root must be present");
        let root = parsed.tree();

        if let Some(source_path) = self.source_path {
            match schema_store
                .resolve_source_schema_from_ast(&root, Some(Either::Right(source_path)))
                .await
            {
                Ok(Some(SourceSchema {
                    root_schema: Some(root_schema),
                    ..
                })) => {
                    if let Some(new_toml_version) = root_schema.toml_version() {
                        toml_version = new_toml_version;
                    }
                }
                Err((error, _)) => {
                    return Err(error.into());
                }
                _ => {}
            }
        }

        Ok(toml_version)
    }

    pub(crate) fn try_to_document(
        &self,
        root: tombi_ast::Root,
        toml_version: TomlVersion,
    ) -> Result<tombi_document::Document, crate::de::Error> {
        // Convert the AST to a document tree
        let (document_tree, errors) = root.into_document_tree_and_errors(toml_version).into();

        // Check for errors during document tree construction
        if !errors.is_empty() {
            return Err(errors.into());
        }

        // Convert to a Document
        Ok(document_tree.into_document(toml_version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, TimeZone, Utc};
    use indexmap::{indexmap, IndexMap};
    use serde::Deserialize;
    use tombi_test_lib::project_root_path;

    #[tokio::test]
    async fn test_deserialize_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Test {
            int: i32,
            float: f64,
            string: String,
            bool: bool,
            opt: Option<String>,
        }

        let toml = r#"
int = 42
float = 3.141592653589793
string = "hello"
bool = true
opt = "optional"
"#;

        let expected = Test {
            int: 42,
            float: std::f64::consts::PI,
            string: "hello".to_string(),
            bool: true,
            opt: Some("optional".to_string()),
        };

        let result: Test = from_str_async(toml)
            .await
            .expect("TOML deserialization failed");
        pretty_assertions::assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_deserialize_nested_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Nested {
            value: String,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        struct Test {
            nested: Nested,
            simple_value: i32,
        }

        let toml = r#"
simple_value = 42

[nested]
value = "nested value"
"#;

        let expected = Test {
            nested: Nested {
                value: "nested value".to_string(),
            },
            simple_value: 42,
        };

        let result: Test = from_str_async(toml)
            .await
            .expect("TOML deserialization failed");
        pretty_assertions::assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_deserialize_array() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct SimpleArrayTest {
            values: Vec<i32>,
        }

        let toml = r#"values = [1, 2, 3]"#;

        let expected = SimpleArrayTest {
            values: vec![1, 2, 3],
        };

        let result: SimpleArrayTest = from_str_async(toml)
            .await
            .expect("TOML deserialization failed");
        pretty_assertions::assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_deserialize_map() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct MapTest {
            string_map: IndexMap<String, String>,
            int_map: IndexMap<String, i32>,
        }

        let toml = r#"
[string_map]
key1 = "value1"
key2 = "value2"

[int_map]
one = 1
two = 2
three = 3
"#;

        let expected = MapTest {
            string_map: indexmap! {
                "key1".to_string() => "value1".to_string(),
                "key2".to_string() => "value2".to_string(),
            },
            int_map: indexmap! {
                "one".to_string() => 1,
                "two".to_string() => 2,
                "three".to_string() => 3,
            },
        };

        let result: MapTest = from_str_async(toml)
            .await
            .expect("TOML deserialization failed");
        pretty_assertions::assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_deserialize_enum() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum SimpleEnum {
            Variant1,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        struct EnumTest {
            enum_value: SimpleEnum,
        }

        let toml = r#"enum_value = "Variant1""#;

        let expected = EnumTest {
            enum_value: SimpleEnum::Variant1,
        };

        let result: EnumTest = from_str_async(toml)
            .await
            .expect("TOML deserialization failed");
        pretty_assertions::assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_deserialize_datetime() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct DateTimeTest {
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }

        let toml = r#"
created_at = "2023-05-15T10:30:00Z"
updated_at = "2023-07-20T14:45:30Z"
"#;

        let expected = DateTimeTest {
            created_at: Utc.with_ymd_and_hms(2023, 5, 15, 10, 30, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2023, 7, 20, 14, 45, 30).unwrap(),
        };

        let result: DateTimeTest = from_str_async(toml)
            .await
            .expect("TOML deserialization failed");
        pretty_assertions::assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_deserialize_option() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct OptionTest {
            some: Option<String>,
            none: Option<String>,
        }

        let toml = r#"some = "optional""#;

        let expected = OptionTest {
            some: Some("optional".to_string()),
            none: None,
        };

        let result: OptionTest = from_str_async(toml)
            .await
            .expect("TOML deserialization failed");
        pretty_assertions::assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_deserialize_empty_containers() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct EmptyContainers {
            empty_array: Vec<i32>,
            empty_map: IndexMap<String, String>,
        }

        let toml = r#"
empty_array = []
empty_map = {}
"#;

        let expected = EmptyContainers {
            empty_array: vec![],
            empty_map: IndexMap::new(),
        };

        let result: EmptyContainers = from_str_async(toml)
            .await
            .expect("TOML deserialization failed");
        pretty_assertions::assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_deserialize_special_characters() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct SpecialChars {
            newlines: String,
            quotes: String,
            unicode: String,
            escape_chars: String,
        }

        let toml = r#"
newlines = "line1\nline2\nline3"
quotes = "\"quoted\""
unicode = "日本語の文字列"
escape_chars = "\\t\\n\\r\\\""
"#;

        let expected = SpecialChars {
            newlines: "line1\nline2\nline3".to_string(),
            quotes: "\"quoted\"".to_string(),
            unicode: "日本語の文字列".to_string(),
            escape_chars: "\\t\\n\\r\\\"".to_string(),
        };

        let result: SpecialChars = from_str_async(toml)
            .await
            .expect("TOML deserialization failed");
        pretty_assertions::assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_deserialize_numeric_boundaries() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct NumericBoundaries {
            min_i32: i32,
            max_i32: i32,
            min_f64: f64,
            max_f64: f64,
            zero: f64,
            negative_zero: f64,
        }

        let toml = r#"
min_i32 = -2147483648
max_i32 = 2147483647
min_f64 = -1.7976931348623157e308
max_f64 = 1.7976931348623157e308
zero = 0.0
negative_zero = -0.0
"#;

        let expected = NumericBoundaries {
            min_i32: i32::MIN,
            max_i32: i32::MAX,
            min_f64: f64::MIN,
            max_f64: f64::MAX,
            zero: 0.0,
            negative_zero: -0.0,
        };

        let result: NumericBoundaries = from_str_async(toml)
            .await
            .expect("TOML deserialization failed");
        pretty_assertions::assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_deserialize_complex_nested() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Inner {
            value: String,
            numbers: Vec<i32>,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        struct Middle {
            inner: Inner,
            map: IndexMap<String, Inner>,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        struct ComplexNested {
            middle: Middle,
            array_of_maps: Vec<IndexMap<String, String>>,
        }

        let toml = r#"
[middle.inner]
value = "nested value"
numbers = [1, 2, 3]

[middle.map.key1]
value = "value1"
numbers = [4, 5, 6]

[middle.map.key2]
value = "value2"
numbers = [7, 8, 9]

[[array_of_maps]]
key1 = "value1"
key2 = "value2"

[[array_of_maps]]
key3 = "value3"
key4 = "value4"
"#;

        let expected = ComplexNested {
            middle: Middle {
                inner: Inner {
                    value: "nested value".to_string(),
                    numbers: vec![1, 2, 3],
                },
                map: indexmap! {
                    "key1".to_string() => Inner {
                        value: "value1".to_string(),
                        numbers: vec![4, 5, 6],
                    },
                    "key2".to_string() => Inner {
                        value: "value2".to_string(),
                        numbers: vec![7, 8, 9],
                    },
                },
            },
            array_of_maps: vec![
                indexmap! {
                    "key1".to_string() => "value1".to_string(),
                    "key2".to_string() => "value2".to_string(),
                },
                indexmap! {
                    "key3".to_string() => "value3".to_string(),
                    "key4".to_string() => "value4".to_string(),
                },
            ],
        };

        let result: ComplexNested = from_str_async(toml)
            .await
            .expect("TOML deserialization failed");
        pretty_assertions::assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_deserialize_mixed_type_array() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct MixedTypeArray {
            mixed: Vec<MixedType>,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(untagged)]
        enum MixedType {
            Integer(i32),
            Float(f64),
            String(String),
            Boolean(bool),
        }

        let toml = r#"
mixed = [42, 3.02, "hello", true]
"#;

        let expected = MixedTypeArray {
            mixed: vec![
                MixedType::Integer(42),
                MixedType::Float(3.02),
                MixedType::String("hello".to_string()),
                MixedType::Boolean(true),
            ],
        };

        let result: MixedTypeArray = from_str_async(toml)
            .await
            .expect("TOML deserialization failed");
        pretty_assertions::assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_deserialize_default_values() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct DefaultValues {
            #[serde(default)]
            optional_string: String,
            #[serde(default = "default_i32")]
            optional_i32: i32,
            #[serde(default = "default_vec")]
            optional_vec: Vec<String>,
        }

        fn default_i32() -> i32 {
            42
        }

        fn default_vec() -> Vec<String> {
            vec!["default".to_string()]
        }

        let toml = r#"
optional_string = "provided"
"#;

        let expected = DefaultValues {
            optional_string: "provided".to_string(),
            optional_i32: 42,
            optional_vec: vec!["default".to_string()],
        };

        let result: DefaultValues = from_str_async(toml)
            .await
            .expect("TOML deserialization failed");
        pretty_assertions::assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_empty_tombi_config() {
        tombi_test_lib::init_tracing();
        let toml = r#""#;

        let config: tombi_config::Config = from_str_async(toml)
            .await
            .expect("TOML deserialization failed");

        pretty_assertions::assert_eq!(config, tombi_config::Config::default());
    }

    #[tokio::test]
    async fn test_deserialize_actual_tombi_config() {
        let config_path = project_root_path().join("tombi.toml");
        let config = crate::config::from_str(
            &std::fs::read_to_string(&config_path).unwrap(),
            &config_path,
        )
        .expect("Failed to parse tombi.toml");

        // Verify the parsed values
        pretty_assertions::assert_eq!(
            config.toml_version,
            Some(tombi_toml_version::TomlVersion::V1_0_0)
        );
        assert!(config.format.is_some());
        assert!(config.lint.is_some());
        assert!(config.lsp().is_some());
        assert!(config.schema.is_some());
        assert!(config.schemas.is_some());

        let schema = config.schema.unwrap();
        pretty_assertions::assert_eq!(
            schema.enabled,
            Some(tombi_config::BoolDefaultTrue::default())
        );

        let schemas = config.schemas.unwrap();
        pretty_assertions::assert_eq!(schemas.len(), 2);

        // Verify the first schema
        let first_schema = &schemas[0];
        pretty_assertions::assert_eq!(first_schema.path(), "schemas/type-test.schema.json");
    }
}
