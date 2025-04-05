mod array;
mod boolean;
mod date_time;
mod float;
mod integer;
mod string;
mod table;

pub use array::{Array, ArrayKind};
pub use boolean::Boolean;
pub use date_time::{LocalDate, LocalDateTime, LocalTime, OffsetDateTime, TimeZoneOffset};
pub use float::Float;
pub use integer::{Integer, IntegerKind};
pub use string::{String, StringKind};
pub use table::{Table, TableKind};

use crate::key::Key;
use crate::IntoDocument;
use serde::de::Error as SerdeError;
use std::string::String as StdString;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(Boolean),
    Integer(Integer),
    Float(Float),
    String(String),
    OffsetDateTime(OffsetDateTime),
    LocalDateTime(LocalDateTime),
    LocalDate(LocalDate),
    LocalTime(LocalTime),
    Array(Array),
    Table(Table),
}

#[derive(Debug)]
pub struct Error(StdString);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl SerdeError for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self(msg.to_string())
    }
}

impl<'de> serde::de::IntoDeserializer<'de, Error> for &'de Key {
    type Deserializer = serde::de::value::StrDeserializer<'de, Error>;

    fn into_deserializer(self) -> Self::Deserializer {
        serde::de::value::StrDeserializer::new(self.value())
    }
}

pub struct ValueDeserializer<'de> {
    value: &'de Value,
}

impl<'de> ValueDeserializer<'de> {
    pub fn new(value: &'de Value) -> Self {
        Self { value }
    }
}

impl<'de> serde::de::Deserializer<'de> for ValueDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Boolean(b) => visitor.visit_bool(b.value()),
            Value::Integer(i) => visitor.visit_i64(i.value()),
            Value::Float(f) => visitor.visit_f64(f.value()),
            Value::String(s) => visitor.visit_str(s.value()),
            Value::Array(a) => {
                let seq = a.values();
                visitor.visit_seq(serde::de::value::SeqDeserializer::new(seq.iter()))
            }
            Value::Table(t) => {
                let map = t.key_values();
                visitor.visit_map(serde::de::value::MapDeserializer::new(map.iter()))
            }
            _ => Err(Error::custom("Unsupported value type")),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Boolean(b) => visitor.visit_bool(b.value()),
            _ => Err(Error::custom("Expected boolean")),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Integer(i) => visitor.visit_i8(i.value() as i8),
            _ => Err(Error::custom("Expected integer")),
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Integer(i) => visitor.visit_i16(i.value() as i16),
            _ => Err(Error::custom("Expected integer")),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Integer(i) => visitor.visit_i32(i.value() as i32),
            _ => Err(Error::custom("Expected integer")),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Integer(i) => visitor.visit_i64(i.value()),
            _ => Err(Error::custom("Expected integer")),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Integer(i) => visitor.visit_u8(i.value() as u8),
            _ => Err(Error::custom("Expected integer")),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Integer(i) => visitor.visit_u16(i.value() as u16),
            _ => Err(Error::custom("Expected integer")),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Integer(i) => visitor.visit_u32(i.value() as u32),
            _ => Err(Error::custom("Expected integer")),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Integer(i) => visitor.visit_u64(i.value() as u64),
            _ => Err(Error::custom("Expected integer")),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Float(f) => visitor.visit_f32(f.value() as f32),
            _ => Err(Error::custom("Expected float")),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Float(f) => visitor.visit_f64(f.value()),
            _ => Err(Error::custom("Expected float")),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::String(s) => {
                let chars: Vec<char> = s.value().chars().collect();
                if chars.len() == 1 {
                    visitor.visit_char(chars[0])
                } else {
                    Err(Error::custom("Expected single character"))
                }
            }
            _ => Err(Error::custom("Expected string")),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::String(s) => visitor.visit_str(s.value()),
            _ => Err(Error::custom("Expected string")),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::String(s) => visitor.visit_string(s.value().to_string()),
            _ => Err(Error::custom("Expected string")),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::String(s) => visitor.visit_bytes(s.value().as_bytes()),
            _ => Err(Error::custom("Expected string")),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::String(s) => visitor.visit_byte_buf(s.value().as_bytes().to_vec()),
            _ => Err(Error::custom("Expected string")),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Array(a) => {
                let seq = a.values();
                visitor.visit_seq(serde::de::value::SeqDeserializer::new(seq.iter()))
            }
            _ => Err(Error::custom("Expected array")),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Array(a) => {
                let seq = a.values();
                visitor.visit_seq(serde::de::value::SeqDeserializer::new(seq.iter()))
            }
            _ => Err(Error::custom("Expected array")),
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Array(a) => {
                let seq = a.values();
                visitor.visit_seq(serde::de::value::SeqDeserializer::new(seq.iter()))
            }
            _ => Err(Error::custom("Expected array")),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Table(t) => {
                let map = t.key_values();
                visitor.visit_map(serde::de::value::MapDeserializer::new(map.iter()))
            }
            _ => Err(Error::custom("Expected table")),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Table(t) => {
                let map = t.key_values();
                visitor.visit_map(serde::de::value::MapDeserializer::new(map.iter()))
            }
            _ => Err(Error::custom("Expected table")),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::String(s) => {
                visitor.visit_enum(serde::de::value::StrDeserializer::new(s.value()))
            }
            Value::Table(t) => {
                let map = t.key_values();
                visitor.visit_map(serde::de::value::MapDeserializer::new(map.iter()))
            }
            _ => Err(Error::custom("Expected string or table")),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::String(s) => visitor.visit_str(s.value()),
            _ => Err(Error::custom("Expected string")),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

impl<'de> serde::de::IntoDeserializer<'de, Error> for &'de Value {
    type Deserializer = ValueDeserializer<'de>;

    fn into_deserializer(self) -> Self::Deserializer {
        ValueDeserializer::new(self)
    }
}

impl IntoDocument<Value> for document_tree::Value {
    fn into_document(self, toml_version: crate::TomlVersion) -> Value {
        match self {
            document_tree::Value::Boolean(value) => Value::Boolean(value.into()),
            document_tree::Value::Integer(value) => Value::Integer(value.into()),
            document_tree::Value::Float(value) => Value::Float(value.into()),
            document_tree::Value::String(value) => Value::String(value.into()),
            document_tree::Value::OffsetDateTime(value) => Value::OffsetDateTime(value.into()),
            document_tree::Value::LocalDateTime(value) => Value::LocalDateTime(value.into()),
            document_tree::Value::LocalDate(value) => Value::LocalDate(value.into()),
            document_tree::Value::LocalTime(value) => Value::LocalTime(value.into()),
            document_tree::Value::Array(value) => Value::Array(value.into_document(toml_version)),
            document_tree::Value::Table(value) => Value::Table(value.into_document(toml_version)),
            document_tree::Value::Incomplete { .. } => {
                unreachable!("Incomplete value should not be converted to document")
            }
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Boolean(value) => value.serialize(serializer),
            Value::Integer(value) => value.serialize(serializer),
            Value::Float(value) => value.serialize(serializer),
            Value::String(value) => value.serialize(serializer),
            Value::OffsetDateTime(value) => value.serialize(serializer),
            Value::LocalDateTime(value) => value.serialize(serializer),
            Value::LocalDate(value) => value.serialize(serializer),
            Value::LocalTime(value) => value.serialize(serializer),
            Value::Array(value) => value.serialize(serializer),
            Value::Table(value) => value.serialize(serializer),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> serde::de::Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid Value")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Value, E> {
                Ok(Value::Boolean(Boolean::new(v)))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Value, E> {
                Ok(Value::Integer(Integer::new(v)))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Integer(Integer::new(v as i64)))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Value, E> {
                Ok(Value::Float(Float::new(v)))
            }

            fn visit_str<E>(self, v: &str) -> Result<Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::String(String::new(
                    StringKind::BasicString,
                    v.to_string(),
                )))
            }

            fn visit_string<E>(self, v: std::string::String) -> Result<Value, E> {
                Ok(Value::String(String::new(
                    StringKind::BasicString,
                    v.to_string(),
                )))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut vec = Array::new(ArrayKind::ArrayOfTable);
                while let Some(elem) = seq.next_element()? {
                    vec.push(elem);
                }
                Ok(Value::Array(vec))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut index_map = Table::new(TableKind::Table);
                while let Some((key, value)) = map.next_entry()? {
                    index_map.insert(key, value);
                }
                Ok(Value::Table(index_map))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}
