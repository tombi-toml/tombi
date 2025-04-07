mod error;

use formatter::formatter::definitions::FormatDefinitions;
use formatter::FormatOptions;
use schema_store::SchemaStore;
use serde::Serialize;
use toml_version::TomlVersion;

use crate::document::ToTomlString;
pub use error::Error;

/// Serialize the given data structure as a TOML string.
///
/// # Examples
///
/// ```
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Config {
///     ip: String,
///     port: u16,
///     keys: Vec<String>,
/// }
///
/// let config = Config {
///     ip: "127.0.0.1".to_string(),
///     port: 8080,
///     keys: vec!["key1".to_string(), "key2".to_string()],
/// };
///
/// let toml = serde_tombi::to_string(&config);
/// ```
pub fn to_string<T>(value: &T) -> Result<String, crate::ser::Error>
where
    T: Serialize,
{
    Serializer::default().to_string(value)
}

pub async fn to_string_async<T>(value: &T) -> Result<String, crate::ser::Error>
where
    T: Serialize,
{
    Serializer::default().to_string_async(value).await
}

/// Serialize the given data structure as a TOML Document.
pub fn to_document<T>(value: &T) -> Result<crate::Document, crate::ser::Error>
where
    T: Serialize,
{
    Serializer::default().to_document(value)
}

// Actual serializer implementation
#[derive(Default)]
pub struct Serializer<'a> {
    config: Option<&'a config::Config>,
    config_path: Option<&'a std::path::Path>,
    schema_store: Option<&'a schema_store::SchemaStore>,
}

impl<'a> Serializer<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(
        mut self,
        config: &'a config::Config,
        config_path: Option<&'a std::path::Path>,
    ) -> Self {
        self.config = Some(config);
        self.config_path = config_path;
        self
    }

    pub fn with_schema_store(mut self, schema_store: &'a schema_store::SchemaStore) -> Self {
        self.schema_store = Some(schema_store);
        self
    }

    pub fn to_document<T>(&self, value: &T) -> Result<document::Document, crate::ser::Error>
    where
        T: Serialize,
    {
        match value.serialize(&mut ValueSerializer { accessors: &[] }) {
            Ok(Some(document::Value::Table(table))) => Ok(document::Document::from(table)),
            Ok(Some(value)) => Err(crate::ser::Error::RootMustBeTable(value.kind())),
            Ok(None) => Ok(document::Document::new()),
            Err(error) => Err(crate::ser::Error::Serde(error.to_string())),
        }
    }

    pub fn to_string<T>(&self, value: &T) -> Result<String, crate::ser::Error>
    where
        T: Serialize,
    {
        tokio::runtime::Runtime::new()?.block_on(self.to_string_async(value))
    }

    pub async fn to_string_async<T>(&self, value: &T) -> Result<String, crate::ser::Error>
    where
        T: Serialize,
    {
        let document = to_document(value)?;
        let mut toml_text = std::string::String::new();
        document.to_toml_string(&mut toml_text, &[]);

        let format_definitions = FormatDefinitions::default();
        let format_options = FormatOptions::default();

        let schema_store = match self.schema_store {
            Some(schema_store) => schema_store,
            None => &SchemaStore::default(),
        };
        if self.schema_store.is_none() {
            match self.config {
                Some(config) => {
                    schema_store.load_config(&config, self.config_path).await?;
                }
                None => {
                    let (config, config_path) = config::load_with_path()?;
                    schema_store
                        .load_config(&config, config_path.as_deref())
                        .await?;
                }
            };
        }

        let formatter = formatter::Formatter::new(
            TomlVersion::default(),
            format_definitions,
            &format_options,
            None,
            schema_store,
        );

        match formatter.format(&toml_text).await {
            Ok(formatted) => Ok(formatted),
            Err(errors) => {
                tracing::trace!("toml_text:\n{}", toml_text);
                tracing::trace!(?errors);
                unreachable!("Document must be valid TOML.")
            }
        }
    }
}

pub struct ValueSerializer<'a> {
    accessors: &'a [schema_store::Accessor],
}

impl<'a> serde::Serializer for &'a mut ValueSerializer<'a> {
    type Ok = Option<document::Value>;
    type Error = crate::ser::Error;
    type SerializeSeq = SerializeArray<'a>;
    type SerializeTuple = SerializeArray<'a>;
    type SerializeTupleStruct = SerializeArray<'a>;
    type SerializeTupleVariant = SerializeArray<'a>;
    type SerializeMap = SerializeTable<'a>;
    type SerializeStruct = SerializeTable<'a>;
    type SerializeStructVariant = SerializeTable<'a>;

    // Basic type serialization
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Some(document::Value::Boolean(document::Boolean::new(v))))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Some(document::Value::Integer(document::Integer::new(v))))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Some(document::Value::Float(document::Float::new(v))))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Some(document::Value::String(document::String::new(
            document::StringKind::BasicString,
            v.to_string(),
        ))))
    }

    fn serialize_bytes(self, _: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(crate::ser::Error::TomlMustBeUtf8)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(None)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(crate::ser::Error::SerializeUnit)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(crate::ser::Error::SerializeUnitStruct)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match name {
            date_time::OFFSET_DATE_TIME_NEWTYPE_NAME => value
                .serialize(DateTimeSerializer::new(self.accessors))
                .map(|dt| Some(document::Value::OffsetDateTime(dt))),
            date_time::LOCAL_DATE_TIME_NEWTYPE_NAME => value
                .serialize(DateTimeSerializer::new(self.accessors))
                .map(|dt| Some(document::Value::LocalDateTime(dt))),
            date_time::LOCAL_DATE_NEWTYPE_NAME => value
                .serialize(DateTimeSerializer::new(self.accessors))
                .map(|dt| Some(document::Value::LocalDate(dt))),
            date_time::LOCAL_TIME_NEWTYPE_NAME => value
                .serialize(DateTimeSerializer::new(self.accessors))
                .map(|dt| Some(document::Value::LocalTime(dt))),
            _ => value.serialize(self),
        }
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut *self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeArray {
            kind: document::ArrayKind::ArrayOfTable,
            accessors: self.accessors,
            values: match len {
                Some(len) => Vec::with_capacity(len),
                None => Vec::new(),
            },
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(SerializeArray {
            kind: document::ArrayKind::ArrayOfTable,
            accessors: self.accessors,
            values: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(SerializeArray {
            kind: document::ArrayKind::ArrayOfTable,
            accessors: self.accessors,
            values: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(SerializeArray {
            kind: document::ArrayKind::ArrayOfTable,
            accessors: self.accessors,
            values: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SerializeTable {
            accessors: self.accessors,
            key: None,
            key_values: match len {
                Some(len) => Vec::with_capacity(len),
                None => Vec::new(),
            },
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(SerializeTable {
            accessors: self.accessors,
            key: None,
            key_values: Vec::with_capacity(len),
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(SerializeTable {
            accessors: self.accessors,
            key: None,
            key_values: Vec::with_capacity(len),
        })
    }
}

// Sequence serialization
pub struct SerializeArray<'a> {
    kind: document::ArrayKind,
    accessors: &'a [schema_store::Accessor],
    values: Vec<document::Value>,
}

impl<'a> serde::ser::SerializeSeq for SerializeArray<'a> {
    type Ok = Option<document::Value>;
    type Error = crate::ser::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let Some(mut value) = value.serialize(&mut ValueSerializer {
            accessors: self.accessors,
        })?
        else {
            let mut accessors = self.accessors.to_vec();
            accessors.push(schema_store::Accessor::Index(self.values.len()));

            return Err(crate::ser::Error::ArrayValueRequired(
                schema_store::Accessors::new(accessors),
            ));
        };
        match &mut value {
            document::Value::Boolean(_)
            | document::Value::Integer(_)
            | document::Value::Float(_)
            | document::Value::String(_)
            | document::Value::LocalDate(_)
            | document::Value::LocalDateTime(_)
            | document::Value::LocalTime(_)
            | document::Value::OffsetDateTime(_) => {
                self.kind = document::ArrayKind::Array;
            }
            document::Value::Array(array) => {
                if self.kind == document::ArrayKind::Array {
                    let array_kind = array.kind_mut();
                    *array_kind = document::ArrayKind::Array;
                }
            }
            document::Value::Table(table) => {
                if self.kind == document::ArrayKind::Array {
                    let table_kind = table.kind_mut();
                    *table_kind = document::TableKind::InlineTable;
                }
            }
        }
        self.values.push(value);

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut array = document::Array::new(self.kind);
        for value in self.values {
            array.push(value);
        }
        Ok(Some(document::Value::Array(array)))
    }
}

impl<'a> serde::ser::SerializeTuple for SerializeArray<'a> {
    type Ok = Option<document::Value>;
    type Error = crate::ser::Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl<'a> serde::ser::SerializeTupleStruct for SerializeArray<'a> {
    type Ok = Option<document::Value>;
    type Error = crate::ser::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl<'a> serde::ser::SerializeTupleVariant for SerializeArray<'a> {
    type Ok = Option<document::Value>;
    type Error = crate::ser::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        serde::ser::SerializeSeq::end(self)
    }
}

// Map serialization
pub struct SerializeTable<'a> {
    accessors: &'a [schema_store::Accessor],
    key: Option<document::Key>,
    key_values: Vec<(document::Key, document::Value)>,
}

impl<'a> serde::ser::SerializeMap for SerializeTable<'a> {
    type Ok = Option<document::Value>;
    type Error = crate::ser::Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        // Keys must be converted to strings
        match key.serialize(&mut ValueSerializer {
            accessors: self.accessors,
        }) {
            Ok(Some(document::Value::String(string))) => {
                self.key = Some(document::Key::from(string));
                Ok(())
            }
            Ok(Some(value)) => {
                self.key = None;
                return Err(crate::ser::Error::KeyMustBeString(
                    schema_store::Accessors::new(self.accessors.to_vec()),
                    value.kind(),
                ));
            }
            Ok(None) => {
                self.key = None;
                Err(crate::ser::Error::KeyRequired(
                    schema_store::Accessors::new(self.accessors.to_vec()),
                ))
            }
            Err(error) => {
                self.key = None;
                Err(error.into())
            }
        }
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let Some(key) = self.key.take() else {
            return Err(crate::ser::Error::KeyRequired(
                schema_store::Accessors::new(self.accessors.to_vec()),
            ));
        };
        let Some(value) = value.serialize(&mut ValueSerializer {
            accessors: self.accessors,
        })?
        else {
            self.key = None;
            return Ok(());
        };

        self.key_values.push((key, value));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut table = document::Table::new(document::TableKind::Table);
        for (key, value) in self.key_values {
            table.insert(key, value);
        }
        Ok(Some(document::Value::Table(table)))
    }
}

impl<'a> serde::ser::SerializeStruct for SerializeTable<'a> {
    type Ok = Option<document::Value>;
    type Error = crate::ser::Error;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeMap::serialize_entry(self, key, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        serde::ser::SerializeMap::end(self)
    }
}

impl<'a> serde::ser::SerializeStructVariant for SerializeTable<'a> {
    type Ok = Option<document::Value>;
    type Error = crate::ser::Error;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeStruct::serialize_field(self, key, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        serde::ser::SerializeStruct::end(self)
    }
}

struct DateTimeSerializer<'a, T> {
    accessors: &'a [schema_store::Accessor],
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T> DateTimeSerializer<'a, T> {
    fn new(accessors: &'a [schema_store::Accessor]) -> Self {
        Self {
            accessors,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, T> serde::ser::Serializer for DateTimeSerializer<'a, T>
where
    T: std::str::FromStr,
    T::Err: Into<date_time::parse::Error>,
{
    type Ok = T;
    type Error = crate::ser::Error;
    type SerializeSeq = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_str(self, s: &str) -> Result<Self::Ok, Self::Error> {
        match s.parse::<T>() {
            Ok(value) => Ok(value),
            Err(err) => Err(crate::ser::Error::DateTimeParseError(
                schema_store::Accessors::new(self.accessors.to_vec()),
                err.into(),
            )),
        }
    }

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_some<V>(self, _v: &V) -> Result<Self::Ok, Self::Error>
    where
        V: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_newtype_struct<V>(
        self,
        _name: &'static str,
        _value: &V,
    ) -> Result<Self::Ok, Self::Error>
    where
        V: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn serialize_newtype_variant<V>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &V,
    ) -> Result<Self::Ok, Self::Error>
    where
        V: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        unreachable!()
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        unreachable!()
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        unreachable!()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        unreachable!()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        unreachable!()
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        unreachable!()
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        unreachable!()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, TimeZone, Utc};
    use indexmap::{indexmap, IndexMap};
    use serde::Serialize;
    use test_lib::toml_text_assert_eq;

    #[test]
    fn test_serialize_struct() {
        #[derive(Serialize)]
        struct Test {
            int: i32,
            float: f64,
            string: String,
            bool: bool,
            opt: Option<String>,
        }

        let test = Test {
            int: 42,
            float: 3.14159,
            string: "hello".to_string(),
            bool: true,
            opt: Some("optional".to_string()),
        };

        let toml = to_string(&test).expect("TOML serialization failed");
        let expected = r#"
int = 42
float = 3.14159
string = "hello"
bool = true
opt = "optional"
"#;

        toml_text_assert_eq!(toml, expected);
    }

    #[test]
    fn test_serialize_nested_struct() {
        test_lib::init_tracing();

        #[derive(Serialize)]
        struct Nested {
            value: String,
        }

        #[derive(Serialize)]
        struct Test {
            nested: Nested,
            simple_value: i32,
        }

        let test = Test {
            nested: Nested {
                value: "nested value".to_string(),
            },
            simple_value: 42,
        };

        let toml = to_string(&test).expect("TOML serialization failed");
        let expected = r#"
simple_value = 42

[nested]
value = "nested value"
"#;

        toml_text_assert_eq!(toml, expected);
    }

    #[test]
    fn test_serialize_array() {
        #[derive(Serialize)]
        struct SimpleArrayTest {
            values: Vec<i32>,
        }

        let test = SimpleArrayTest {
            values: vec![1, 2, 3],
        };

        let toml = to_string(&test).expect("TOML serialization failed");
        let expected = r#"values = [1, 2, 3]"#;

        toml_text_assert_eq!(toml, expected);
    }

    #[test]
    fn test_serialize_map() {
        #[derive(Serialize)]
        struct MapTest {
            string_map: IndexMap<String, String>,
            int_map: IndexMap<String, i32>,
        }

        let test = MapTest {
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

        let toml = to_string(&test).expect("TOML serialization failed");
        let expected = r#"
[string_map]
key1 = "value1"
key2 = "value2"

[int_map]
one = 1
two = 2
three = 3
"#
        .strip_prefix("\n")
        .unwrap();

        toml_text_assert_eq!(toml, expected);
    }

    #[test]
    fn test_serialize_enum() {
        #[derive(Serialize)]
        enum SimpleEnum {
            Variant1,
        }

        #[derive(Serialize)]
        struct EnumTest {
            enum_value: SimpleEnum,
        }

        let test = EnumTest {
            enum_value: SimpleEnum::Variant1,
        };

        let toml = to_string(&test).expect("TOML serialization failed");
        let expected = r#"enum_value = "Variant1""#;

        toml_text_assert_eq!(toml, expected);
    }

    #[test]
    fn test_serialize_datetime() {
        #[derive(Serialize)]
        struct DateTimeTest {
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }

        let test = DateTimeTest {
            created_at: Utc.with_ymd_and_hms(2023, 5, 15, 10, 30, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2023, 7, 20, 14, 45, 30).unwrap(),
        };

        let toml = to_string(&test).expect("TOML serialization failed");
        let expected = r#"
created_at = "2023-05-15T10:30:00Z"
updated_at = "2023-07-20T14:45:30Z"
"#;

        toml_text_assert_eq!(toml, expected);
    }

    #[test]
    fn test_serialize_option() {
        #[derive(Serialize)]
        struct OptionTest {
            some: Option<String>,
            none: Option<String>,
        }

        let test = OptionTest {
            some: Some("optional".to_string()),
            none: None,
        };

        let toml = to_string(&test).expect("TOML serialization failed");
        let expected = r#"some = "optional""#;

        toml_text_assert_eq!(toml, expected);
    }
}
