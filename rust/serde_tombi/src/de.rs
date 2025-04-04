use ast::AstNode;
use document::{Document, Value};
use document::{IntoDocument, Key};
use document_tree::IntoDocumentTreeAndErrors;
use itertools::Itertools;
use serde::de::{DeserializeOwned, Deserializer, IntoDeserializer, Visitor};
use std::marker::PhantomData;
use std::ops::Deref;
use toml_version::TomlVersion;

#[derive(Debug)]
pub enum Error {
    Parser(String),
    DocumentTree(String),
    Deserialization(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parser(msg) => write!(f, "Parser error: {}", msg),
            Error::DocumentTree(msg) => write!(f, "Document tree error: {}", msg),
            Error::Deserialization(msg) => write!(f, "Deserialization error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl serde::de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Deserialization(msg.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Deserialize a TOML string into a Rust data structure.
///
/// # Note
///
/// This function is not yet implemented and will return an error.
/// The example below shows the expected usage once implemented.
///
/// # Examples
///
/// ```rust
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
pub fn from_str<T>(s: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let document = parse_str(s)?;
    from_document(document)
}

/// Parse a TOML string into a Document.
pub fn parse_str(s: &str) -> Result<Document> {
    // Parse the source string using the parser
    let parsed = parser::parse(s);

    let errors = parsed.errors(TomlVersion::default()).collect_vec();
    // Check if there are any parsing errors
    if !errors.is_empty() {
        return Err(Error::Parser(
            errors
                .iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<_>>()
                .join(", "),
        ));
    }

    // Cast the parsed result to an AST Root node
    let root = ast::Root::cast(parsed.into_syntax_node())
        .ok_or_else(|| Error::Parser("Failed to cast to AST Root".to_string()))?;

    // Convert the AST to a document tree
    let (document_tree, errors) = root
        .into_document_tree_and_errors(TomlVersion::default())
        .into();

    // Check for errors during document tree construction
    if !errors.is_empty() {
        return Err(Error::DocumentTree(
            errors
                .iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<_>>()
                .join(", "),
        ));
    }

    // Convert to a Document
    Ok(document_tree.into_document(TomlVersion::default()))
}

pub struct DocumentDeserializer<'a>(&'a Document);

impl<'a> From<&'a Document> for DocumentDeserializer<'a> {
    fn from(document: &'a Document) -> Self {
        Self(document)
    }
}

impl<'de> Deserializer<'de> for DocumentDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let table = &self.0;
        let entries: Vec<_> = table
            .key_values()
            .iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        visitor.visit_map(MapAccess::new(entries.into_iter()))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let table = &self.0;
        let entries: Vec<_> = table
            .key_values()
            .iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        visitor.visit_map(MapAccess::new(entries.into_iter()))
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let table = &self.0;
        let key_values: Vec<(String, &Value)> = table
            .key_values()
            .iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        visitor.visit_map(MapAccess::new(key_values.into_iter()))
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let table = &self.0;
        let key_values: Vec<(String, &Value)> = table
            .key_values()
            .iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        visitor.visit_map(MapAccess::new(key_values.into_iter()))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let table = &self.0;
        let key_values: Vec<(String, &Value)> = table
            .key_values()
            .iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        visitor.visit_map(MapAccess::new(key_values.into_iter()))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let table = &self.0;
        let key_values: Vec<(String, &Value)> = table
            .key_values()
            .iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        visitor.visit_map(MapAccess::new(key_values.into_iter()))
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let table = &self.0;
        let key_values: Vec<(String, &Value)> = table
            .key_values()
            .iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        visitor.visit_map(MapAccess::new(key_values.into_iter()))
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let table = &self.0;
        let key_values: Vec<(String, &Value)> = table
            .key_values()
            .iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        visitor.visit_map(MapAccess::new(key_values.into_iter()))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let table = &self.0;
        let key_values: Vec<(String, &Value)> = table
            .key_values()
            .iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        visitor.visit_map(MapAccess::new(key_values.into_iter()))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let table = &self.0;
        let key_values: Vec<(String, &Value)> = table
            .key_values()
            .iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        visitor.visit_map(MapAccess::new(key_values.into_iter()))
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let table = &self.0;
        let key_values: Vec<(String, &Value)> = table
            .key_values()
            .iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        visitor.visit_map(MapAccess::new(key_values.into_iter()))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let table = &self.0;
        let key_values: Vec<(String, &Value)> = table
            .key_values()
            .iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        visitor.visit_map(MapAccess::new(key_values.into_iter()))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let table = &self.0;
        let key_values: Vec<(String, &Value)> = table
            .key_values()
            .iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        visitor.visit_map(MapAccess::new(key_values.into_iter()))
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let table = &self.0;
        let key_values: Vec<(String, &Value)> = table
            .key_values()
            .iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        visitor.visit_map(MapAccess::new(key_values.into_iter()))
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let table = &self.0;
        let key_values: Vec<(String, &Value)> = table
            .key_values()
            .iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        visitor.visit_map(MapAccess::new(key_values.into_iter()))
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let table = &self.0;
        match Value::Table(table.clone()) {
            Value::String(s) => visitor.visit_bytes(s.value().as_bytes()),
            _ => Err(Error::Deserialization("Expected string".to_string())),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let table = &self.0;
        match Value::Table(table.clone()) {
            Value::String(s) => visitor.visit_byte_buf(s.value().as_bytes().to_vec()),
            _ => Err(Error::Deserialization("Expected string".to_string())),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> std::result::Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let table = self.0.deref();
        match Value::Table(table.clone()) {
            Value::Array(a) => {
                let values: Vec<&Value> = a.values().iter().collect();
                visitor.visit_seq(SeqAccess {
                    values: values.into_iter(),
                })
            }
            _ => Err(Error::Deserialization("Expected array".to_string())),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> std::result::Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let table = self.0.deref();
        match Value::Table(table.clone()) {
            Value::Array(a) => {
                let values: Vec<Value> = a.values().into_iter().cloned().collect();
                let values_static: Vec<&'static Value> = values
                    .iter()
                    .map(|v| unsafe { std::mem::transmute(v as &Value) })
                    .collect();
                visitor
                    .visit_seq(SeqAccess {
                        values: values_static.into_iter(),
                    })
                    .map_err(|e| Error::Deserialization(e.to_string()))
            }
            _ => Err(Error::Deserialization("Expected array".to_string())),
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> std::result::Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let table = self.0.deref();
        match Value::Table(table.clone()) {
            Value::Array(a) => {
                let values: Vec<Value> = a.values().into_iter().cloned().collect();
                let values_static: Vec<&'static Value> = values
                    .iter()
                    .map(|v| unsafe { std::mem::transmute(v as &Value) })
                    .collect();
                visitor
                    .visit_seq(SeqAccess {
                        values: values_static.into_iter(),
                    })
                    .map_err(|e| Error::Deserialization(e.to_string()))
            }
            _ => Err(Error::Deserialization("Expected array".to_string())),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Table(t) => {
                let key_values: Vec<(String, &Value)> = t
                    .key_values()
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect();
                visitor.visit_map(MapAccess::new(key_values.into_iter()))
            }
            _ => Err(Error::Deserialization("Expected table".to_string())),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Table(t) => {
                let key_values: Vec<(String, &Value)> = t
                    .key_values()
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect();
                visitor.visit_map(MapAccess::new(key_values.into_iter()))
            }
            _ => Err(Error::Deserialization("Expected table".to_string())),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let table = self.0.deref();
        let value = Value::Table(table.clone());
        match value {
            Value::String(s) => {
                visitor.visit_enum(serde::de::value::StrDeserializer::new(s.value()))
            }
            Value::Table(t) => {
                // TODO: Implement enum deserialization
                Err(Error::Deserialization(
                    "Enum deserialization is not implemented yet".to_string(),
                ))
            }
            _ => Err(Error::Deserialization(
                "Expected string or table".to_string(),
            )),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> std::result::Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let table = self.0.deref();
        let value = Value::Table(table.clone());
        match value {
            Value::String(s) => visitor.visit_str(s.value()),
            _ => Err(Error::Deserialization("Expected string".to_string())),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> std::result::Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

/// Deserialize a Document into a Rust data structure.
pub fn from_document<T>(document: Document) -> Result<T>
where
    T: DeserializeOwned,
{
    T::deserialize(DocumentDeserializer::from(&document))
        .map_err(|e| Error::Deserialization(e.to_string()))
}

impl<'de> IntoDeserializer<'de, Error> for &'de Value {
    type Deserializer = ValueDeserializer<'de>;

    fn into_deserializer(self) -> Self::Deserializer {
        ValueDeserializer(self)
    }
}

struct ValueDeserializer<'de>(&'de Value);

impl<'de> Deserializer<'de> for ValueDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Boolean(b) => visitor.visit_bool(b.value()),
            Value::Integer(i) => visitor.visit_i64(i.value()),
            Value::Float(f) => visitor.visit_f64(f.value()),
            Value::String(s) => visitor.visit_str(s.value()),
            Value::Array(a) => {
                let values: Vec<&Value> = a.values().iter().collect();
                visitor.visit_seq(SeqAccess {
                    values: values.into_iter(),
                })
            }
            Value::Table(t) => {
                let key_values: Vec<(String, &Value)> = t
                    .key_values()
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect();
                visitor.visit_map(MapAccess::new(key_values.into_iter()))
            }
            Value::OffsetDateTime(dt) => visitor.visit_str(&dt.to_string()),
            Value::LocalDateTime(dt) => visitor.visit_str(&dt.to_string()),
            Value::LocalDate(d) => visitor.visit_str(&d.to_string()),
            Value::LocalTime(t) => visitor.visit_str(&t.to_string()),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Boolean(b) => visitor.visit_bool(b.value()),
            _ => Err(Error::Deserialization("Expected boolean".to_string())),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Integer(i) => visitor.visit_i8(i.value() as i8),
            _ => Err(Error::Deserialization("Expected integer".to_string())),
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Integer(i) => visitor.visit_i16(i.value() as i16),
            _ => Err(Error::Deserialization("Expected integer".to_string())),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Integer(i) => visitor.visit_i32(i.value() as i32),
            _ => Err(Error::Deserialization("Expected integer".to_string())),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Integer(i) => visitor.visit_i64(i.value()),
            _ => Err(Error::Deserialization("Expected integer".to_string())),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Integer(i) => visitor.visit_u8(i.value() as u8),
            _ => Err(Error::Deserialization("Expected integer".to_string())),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Integer(i) => visitor.visit_u16(i.value() as u16),
            _ => Err(Error::Deserialization("Expected integer".to_string())),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Integer(i) => visitor.visit_u32(i.value() as u32),
            _ => Err(Error::Deserialization("Expected integer".to_string())),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Integer(i) => visitor.visit_u64(i.value() as u64),
            _ => Err(Error::Deserialization("Expected integer".to_string())),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Float(f) => visitor.visit_f32(f.value() as f32),
            _ => Err(Error::Deserialization("Expected float".to_string())),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Float(f) => visitor.visit_f64(f.value()),
            _ => Err(Error::Deserialization("Expected float".to_string())),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::String(s) => {
                let chars: Vec<char> = s.value().chars().collect();
                if chars.len() == 1 {
                    visitor.visit_char(chars[0])
                } else {
                    Err(Error::Deserialization(
                        "Expected single character".to_string(),
                    ))
                }
            }
            _ => Err(Error::Deserialization("Expected string".to_string())),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::String(s) => visitor.visit_str(s.value()),
            _ => Err(Error::Deserialization("Expected string".to_string())),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::String(s) => visitor.visit_string(s.value().to_string()),
            _ => Err(Error::Deserialization("Expected string".to_string())),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::String(s) => visitor.visit_bytes(s.value().as_bytes()),
            _ => Err(Error::Deserialization("Expected string".to_string())),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::String(s) => visitor.visit_byte_buf(s.value().as_bytes().to_vec()),
            _ => Err(Error::Deserialization("Expected string".to_string())),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Array(a) => {
                let values: Vec<&Value> = a.values().iter().collect();
                visitor.visit_seq(SeqAccess {
                    values: values.into_iter(),
                })
            }
            _ => Err(Error::Deserialization("Expected array".to_string())),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Array(a) => {
                let values: Vec<&Value> = a.values().iter().collect();
                visitor.visit_seq(SeqAccess {
                    values: values.into_iter(),
                })
            }
            _ => Err(Error::Deserialization("Expected array".to_string())),
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Array(a) => {
                let values: Vec<&Value> = a.values().iter().collect();
                visitor.visit_seq(SeqAccess {
                    values: values.into_iter(),
                })
            }
            _ => Err(Error::Deserialization("Expected array".to_string())),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Table(t) => {
                let key_values: Vec<(String, &Value)> = t
                    .key_values()
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect();
                visitor.visit_map(MapAccess::new(key_values.into_iter()))
            }
            _ => Err(Error::Deserialization("Expected table".to_string())),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Table(t) => {
                let key_values: Vec<(String, &Value)> = t
                    .key_values()
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect();
                visitor.visit_map(MapAccess::new(key_values.into_iter()))
            }
            _ => Err(Error::Deserialization("Expected table".to_string())),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::String(s) => {
                visitor.visit_enum(serde::de::value::StrDeserializer::new(s.value()))
            }
            Value::Table(t) => {
                let mut entries = t.key_values().into_iter();
                let (key, value) = entries.next().ok_or_else(|| {
                    Error::Deserialization("Expected at least one key-value pair".to_string())
                })?;

                if entries.next().is_some() {
                    return Err(Error::Deserialization(
                        "Expected exactly one key-value pair".to_string(),
                    ));
                }

                visitor.visit_enum(EnumAccess {
                    key: key.to_string(),
                    value: value,
                })
            }
            _ => Err(Error::Deserialization(
                "Expected string or table".to_string(),
            )),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::String(s) => visitor.visit_str(s.value()),
            _ => Err(Error::Deserialization("Expected string".to_string())),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

struct EnumAccess<'a> {
    key: String,
    value: &'a Value,
}

impl<'de, 'a> serde::de::EnumAccess<'de> for EnumAccess<'a> {
    type Error = Error;
    type Variant = VariantAccess<'a>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let value = seed.deserialize(serde::de::value::StrDeserializer::new(&self.key))?;
        Ok((value, VariantAccess { value: self.value }))
    }
}

struct VariantAccess<'a> {
    value: &'a Value,
}

impl<'de, 'a> serde::de::VariantAccess<'de> for VariantAccess<'a> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        match self.value {
            Value::Table(t) if t.key_values().len() == 0 => Ok(()),
            _ => Err(Error::Deserialization(
                "Expected empty table for unit variant".to_string(),
            )),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(ValueDeserializer(self.value))
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Array(a) => {
                let values: Vec<&Value> = a.values().iter().collect();
                visitor.visit_seq(serde::de::value::SeqDeserializer::<_, Error>::new(
                    values.into_iter(),
                ))
            }
            _ => Err(Error::Deserialization(
                "Expected array for tuple variant".to_string(),
            )),
        }
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Table(t) => {
                let key_values: Vec<(String, &Value)> = t
                    .key_values()
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect();
                visitor
                    .visit_map(serde::de::value::MapDeserializer::<
                        'de,
                        std::vec::IntoIter<(String, &'de Value)>,
                        Error,
                    >::new(key_values.into_iter()))
                    .map_err(|e| Error::Deserialization(e.to_string()))
            }
            _ => Err(Error::Deserialization(
                "Expected table for struct variant".to_string(),
            )),
        }
    }
}

struct MapAccess<'a, 'de: 'a> {
    entries: std::vec::IntoIter<(String, &'a Value)>,
    _phantom: PhantomData<&'de ()>,
}

impl<'a, 'de: 'a> MapAccess<'a, 'de> {
    fn new(entries: std::vec::IntoIter<(String, &'a Value)>) -> Self {
        Self {
            entries,
            _phantom: PhantomData,
        }
    }
}

impl<'a, 'de: 'a> serde::de::MapAccess<'de> for MapAccess<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        match self.entries.next() {
            Some((key, _)) => seed
                .deserialize(serde::de::value::StrDeserializer::new(&key))
                .map(Some),
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        match self.entries.next() {
            Some((_, value)) => seed.deserialize(ValueDeserializer(value)),
            None => Err(Error::Deserialization("Expected value".to_string())),
        }
    }
}

struct SeqAccess<'a> {
    values: std::vec::IntoIter<&'a Value>,
}

impl<'de, 'a> serde::de::SeqAccess<'de> for SeqAccess<'a> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.values.next() {
            Some(value) => seed
                .deserialize(ValueDeserializer(value))
                .map(Some)
                .map_err(|e| Error::Deserialization(e.to_string())),
            None => Ok(None),
        }
    }
}
