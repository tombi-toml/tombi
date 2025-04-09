mod error;

use ast::AstNode;
use document::IntoDocument;
use document_tree::IntoDocumentTreeAndErrors;
pub use error::Error;
use itertools::Itertools;
use schema_store::{SchemaStore, SourceSchema};
use serde::de::DeserializeOwned;
use toml_version::TomlVersion;
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
/// ```no_run
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Config {
///     ip: String,
///     port: u16,
///     keys: Vec<String>,
/// }
///
/// let toml = r#"
/// ip = "127.0.0.1"
/// port = 8080
/// keys = ["key1", "key2"]
/// "#;
///
/// let config: Config = serde_tombi::from_str(toml).unwrap();
/// ```
pub fn from_str<T>(s: &str) -> Result<T, crate::de::Error>
where
    T: DeserializeOwned,
{
    Deserializer::new().from_str(s)
}

pub async fn from_str_async<T>(s: &str) -> Result<T, crate::de::Error>
where
    T: DeserializeOwned,
{
    Deserializer::new().from_str_async(s).await
}

pub fn from_document<T>(document: document::Document) -> Result<T, crate::de::Error>
where
    T: DeserializeOwned,
{
    Deserializer::new().from_document(document)
}

// Actual deserializer implementation
#[derive(TypedBuilder)]
pub struct Deserializer<'de> {
    #[builder(default, setter(into, strip_option))]
    config: Option<&'de ::config::Config>,

    #[builder(default, setter(into, strip_option))]
    config_path: Option<&'de std::path::Path>,

    #[builder(default, setter(into, strip_option))]
    source_path: Option<&'de std::path::Path>,

    #[builder(default, setter(into, strip_option))]
    schema_store: Option<&'de schema_store::SchemaStore>,
}

impl<'de> Deserializer<'de> {
    pub fn new() -> Self {
        Self {
            config: None,
            config_path: None,
            source_path: None,
            schema_store: None,
        }
    }

    pub fn from_str<T>(&self, s: &str) -> Result<T, crate::de::Error>
    where
        T: DeserializeOwned,
    {
        tokio::runtime::Runtime::new()?.block_on(self.from_str_async(s))
    }

    pub async fn from_str_async<T>(&self, s: &str) -> Result<T, crate::de::Error>
    where
        T: DeserializeOwned,
    {
        from_document(self.try_to_document(s).await?)
    }

    pub fn from_document<T>(&self, document: document::Document) -> Result<T, crate::de::Error>
    where
        T: DeserializeOwned,
    {
        Ok(T::deserialize(&document)?)
    }

    async fn try_to_document(&self, s: &str) -> Result<document::Document, crate::de::Error> {
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

        if let Some(source_path) = self.source_path {
            if let Some(SourceSchema {
                root_schema: Some(root_schema),
                ..
            }) = schema_store
                .try_get_source_schema_from_path(source_path)
                .await?
            {
                if let Some(new_toml_version) = root_schema.toml_version() {
                    toml_version = new_toml_version;
                }
            }
        }

        // Parse the source string using the parser
        let parsed = parser::parse(s);

        let errors = parsed.errors(toml_version).collect_vec();
        // Check if there are any parsing errors
        if !errors.is_empty() {
            return Err(crate::de::Error::Parser(
                parsed.into_errors(toml_version).collect_vec(),
            ));
        }

        // Cast the parsed result to an AST Root node
        let root = ast::Root::cast(parsed.into_syntax_node()).expect("AST Root must be present");

        // Convert the AST to a document tree
        let (document_tree, errors) = root.into_document_tree_and_errors(toml_version).into();

        // Check for errors during document tree construction
        if !errors.is_empty() {
            return Err(crate::de::Error::DocumentTree(errors));
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
    use test_lib::project_root;

    #[test]
    fn test_deserialize_struct() {
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

        let result: Test = from_str(toml).expect("TOML deserialization failed");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_deserialize_nested_struct() {
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

        let result: Test = from_str(toml).expect("TOML deserialization failed");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_deserialize_array() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct SimpleArrayTest {
            values: Vec<i32>,
        }

        let toml = r#"values = [1, 2, 3]"#;

        let expected = SimpleArrayTest {
            values: vec![1, 2, 3],
        };

        let result: SimpleArrayTest = from_str(toml).expect("TOML deserialization failed");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_deserialize_map() {
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

        let result: MapTest = from_str(toml).expect("TOML deserialization failed");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_deserialize_enum() {
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

        let result: EnumTest = from_str(toml).expect("TOML deserialization failed");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_deserialize_datetime() {
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

        let result: DateTimeTest = from_str(toml).expect("TOML deserialization failed");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_deserialize_option() {
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

        let result: OptionTest = from_str(toml).expect("TOML deserialization failed");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_deserialize_empty_containers() {
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

        let result: EmptyContainers = from_str(toml).expect("TOML deserialization failed");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_deserialize_special_characters() {
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

        let result: SpecialChars = from_str(toml).expect("TOML deserialization failed");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_deserialize_numeric_boundaries() {
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

        let result: NumericBoundaries = from_str(toml).expect("TOML deserialization failed");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_deserialize_complex_nested() {
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

        let result: ComplexNested = from_str(toml).expect("TOML deserialization failed");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_deserialize_mixed_type_array() {
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
mixed = [42, 3.14, "hello", true]
"#;

        let expected = MixedTypeArray {
            mixed: vec![
                MixedType::Integer(42),
                MixedType::Float(3.14),
                MixedType::String("hello".to_string()),
                MixedType::Boolean(true),
            ],
        };

        let result: MixedTypeArray = from_str(toml).expect("TOML deserialization failed");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_deserialize_default_values() {
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

        let result: DefaultValues = from_str(toml).expect("TOML deserialization failed");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_tombi_config() {
        let toml = r#""#;

        let config: config::Config = from_str(toml).expect("TOML deserialization failed");

        pretty_assertions::assert_eq!(config, config::Config::default());
    }

    #[test]
    fn test_deserialize_actual_tombi_config() {
        let config_path = project_root().join("tombi.toml");
        let config_str = std::fs::read_to_string(&config_path).expect("Failed to read tombi.toml");

        let result: config::Config = from_str(&config_str).expect("Failed to parse tombi.toml");

        // Verify the parsed values
        assert_eq!(
            result.toml_version,
            Some(toml_version::TomlVersion::V1_1_0_Preview)
        );
        assert_eq!(result.exclude, Some(vec!["node_modules/**/*".to_string()]));
        assert!(result.format.is_some());
        assert!(result.lint.is_some());
        assert!(result.server.is_some());
        assert!(result.schema.is_some());
        assert!(result.schemas.is_some());

        let schema = result.schema.unwrap();
        assert_eq!(schema.enabled, Some(config::BoolDefaultTrue::default()));

        let schemas = result.schemas.unwrap();
        assert_eq!(schemas.len(), 5);

        // Verify the first schema
        let first_schema = &schemas[0];
        assert_eq!(first_schema.path(), "tombi.schema.json");
        assert_eq!(first_schema.include(), &["tombi.toml"]);

        // Verify the last schema
        let last_schema = &schemas[4];
        assert_eq!(last_schema.path(), "schemas/partial-taskipy.schema.json");
        assert_eq!(last_schema.include(), &["pyproject.toml"]);
        assert_eq!(last_schema.root_keys(), Some("tool.taskipy"));
    }
}
