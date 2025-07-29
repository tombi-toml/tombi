mod node;
mod parser;

pub use node::{ArrayNode, BoolNode, NullNode, NumberNode, ObjectNode, StringNode, ValueNode};
pub use parser::{parse, Error as ParserError};
pub use tombi_json_value::{Number, Object, Value};
pub use tombi_text::Range;

use serde::de::{
    self, DeserializeOwned, Deserializer as SerdeDeserializer, IntoDeserializer, Visitor,
};
use std::fmt;
use std::io::{self, Read};
use std::marker::PhantomData;

/// Error that can occur when deserializing JSON
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// JSON parsing error
    #[error(transparent)]
    Parser(#[from] ParserError),
    /// Custom error message
    #[error("{0}")]
    Custom(String),
    /// IO error
    #[error(transparent)]
    Io(#[from] io::Error),
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

/// Deserialize an instance of type Tree from a string of JSON text
pub fn from_str<T>(s: &str) -> Result<T, crate::Error>
where
    T: DeserializeOwned,
{
    let value_node = parse(s)?;
    from_value_node(value_node)
}

/// Deserialize a ValueNode from a reader
pub fn from_reader<R, T>(reader: R) -> Result<T, Error>
where
    R: Read,
    T: DeserializeOwned,
{
    // Read the entire content from the reader
    let mut buffer = String::new();
    let mut reader = reader;
    reader.read_to_string(&mut buffer)?;

    // Parse the JSON string into a Tree
    from_str(&buffer)
}

// TreeDeserializer that implements serde::Deserializer
pub struct ValueNodeDeserializer<'de> {
    node: ValueNode,
    _marker: PhantomData<&'de ()>,
}

impl ValueNodeDeserializer<'_> {
    pub fn new(node: ValueNode) -> Self {
        ValueNodeDeserializer {
            node,
            _marker: PhantomData,
        }
    }
}

impl<'de> SerdeDeserializer<'de> for ValueNodeDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &self.node {
            ValueNode::Null(_) => visitor.visit_unit(),
            ValueNode::Bool(node) => visitor.visit_bool(node.value),
            ValueNode::Number(node) => {
                if let Some(i) = node.value.as_i64() {
                    visitor.visit_i64(i)
                } else if let Some(f) = node.value.as_f64() {
                    visitor.visit_f64(f)
                } else {
                    Err(Error::Custom("invalid number value".to_string()))
                }
            }
            ValueNode::String(node) => visitor.visit_string(node.value.clone()),
            ValueNode::Array(_) => self.deserialize_seq(visitor),
            ValueNode::Object(_) => self.deserialize_map(visitor),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &self.node {
            ValueNode::Bool(node) => visitor.visit_bool(node.value),
            _ => Err(Error::Custom(format!(
                "invalid type: expected bool, found {:?}",
                self.node
            ))),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &self.node {
            ValueNode::Number(ref n) => {
                if let Some(i) = n.value.as_i64() {
                    visitor.visit_i64(i)
                } else if let Some(f) = n.value.as_f64() {
                    visitor.visit_i64(f as i64)
                } else {
                    Err(Error::Custom(format!(
                        "invalid type: expected i64, found {:?}",
                        self.node
                    )))
                }
            }
            _ => Err(Error::Custom(format!(
                "invalid type: expected i64, found {:?}",
                self.node
            ))),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &self.node {
            ValueNode::Number(ref n) => {
                if let Some(i) = n.value.as_i64() {
                    if i >= 0 {
                        visitor.visit_u64(i as u64)
                    } else {
                        Err(Error::Custom(format!(
                            "invalid value: negative number for u64: {i}"
                        )))
                    }
                } else if let Some(f) = n.value.as_f64() {
                    if f >= 0.0 {
                        visitor.visit_u64(f as u64)
                    } else {
                        Err(Error::Custom(format!(
                            "invalid value: negative number for u64: {f}"
                        )))
                    }
                } else {
                    Err(Error::Custom(format!(
                        "invalid type: expected u64, found {:?}",
                        self.node
                    )))
                }
            }
            _ => Err(Error::Custom(format!(
                "invalid type: expected u64, found {:?}",
                self.node
            ))),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_f64(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &self.node {
            ValueNode::Number(ref n) => {
                if let Some(f) = n.value.as_f64() {
                    visitor.visit_f64(f)
                } else if let Some(i) = n.value.as_i64() {
                    visitor.visit_f64(i as f64)
                } else {
                    Err(Error::Custom(format!(
                        "invalid type: expected f64, found {:?}",
                        self.node
                    )))
                }
            }
            _ => Err(Error::Custom(format!(
                "invalid type: expected f64, found {:?}",
                self.node
            ))),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &self.node {
            ValueNode::String(ref s) => {
                let mut chars = s.value.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => visitor.visit_char(c),
                    _ => Err(Error::Custom(format!(
                        "invalid value: expected single character string, found {:?}",
                        self.node
                    ))),
                }
            }
            _ => Err(Error::Custom(format!(
                "invalid type: expected char, found {:?}",
                self.node
            ))),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &self.node {
            ValueNode::String(s) => visitor.visit_string(s.value.clone()),
            _ => Err(Error::Custom(format!(
                "invalid type: expected string, found {:?}",
                self.node
            ))),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &self.node {
            ValueNode::Array(arr) => {
                let mut bytes = Vec::with_capacity(arr.len());
                for item in &arr.items {
                    match item {
                        ValueNode::Number(n) => {
                            if let Some(i) = n.value.as_i64() {
                                if (0..=255).contains(&i) {
                                    bytes.push(i as u8);
                                } else {
                                    return Err(Error::Custom(format!(
                                        "invalid value for byte: {i}, should be 0-255"
                                    )));
                                }
                            } else {
                                return Err(Error::Custom(format!(
                                    "invalid value for byte: {n:?}"
                                )));
                            }
                        }
                        _ => {
                            return Err(Error::Custom(format!(
                                "invalid type for byte array element: expected number, found {item:?}"
                            )));
                        }
                    }
                }
                visitor.visit_bytes(&bytes)
            }
            ValueNode::String(s) => visitor.visit_bytes(s.value.as_bytes()),
            _ => Err(Error::Custom(format!(
                "invalid type: expected array or string, found {:?}",
                self.node
            ))),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.node {
            ValueNode::Null(_) => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.node {
            ValueNode::Null(_) => visitor.visit_unit(),
            _ => Err(Error::Custom(format!(
                "invalid type: expected null, found {:?}",
                self.node
            ))),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
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

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.node {
            ValueNode::Array(array) => {
                let seq_access = SeqAccess {
                    items: array.items.into_iter(),
                };
                visitor.visit_seq(seq_access)
            }
            _ => Err(Error::Custom(format!(
                "invalid type: expected array, found {:?}",
                self.node
            ))),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.node {
            ValueNode::Object(object_node) => {
                let map_access = MapAccess {
                    properties: object_node.properties.into_iter(),
                    key: None,
                    value: None,
                };
                visitor.visit_map(map_access)
            }
            _ => Err(Error::Custom(format!(
                "invalid type: expected object, found {:?}",
                self.node
            ))),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.node {
            ValueNode::String(s) => visitor.visit_enum(TreeEnumAccess::new(s.value, None)),
            ValueNode::Object(obj) if obj.len() == 1 => {
                let Some((variant, value_node)) = obj.properties.into_iter().next() else {
                    return Err(Error::Custom(
                        "invalid type: expected enum, found object with no properties".to_string(),
                    ));
                };

                visitor.visit_enum(TreeEnumAccess {
                    variant: variant.value,
                    value: Some(value_node),
                })
            }
            _ => Err(Error::Custom(format!(
                "invalid type: expected string or map with single key, found {:?}",
                self.node
            ))),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct SeqAccess {
    items: std::vec::IntoIter<ValueNode>,
}

impl<'de> serde::de::SeqAccess<'de> for SeqAccess {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let Some(item) = self.items.next() else {
            return Ok(None);
        };

        seed.deserialize(ValueNodeDeserializer::new(item)).map(Some)
    }
}

struct MapAccess {
    properties: indexmap::map::IntoIter<StringNode, ValueNode>,
    key: Option<String>,
    value: Option<ValueNode>,
}

impl<'de> serde::de::MapAccess<'de> for MapAccess {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if let Some((key, value)) = self.properties.next() {
            self.key = Some(key.value.clone());
            let key = seed.deserialize(ValueNodeDeserializer::new(ValueNode::String(key)))?;
            self.value = Some(value);

            Ok(Some(key))
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        if let Some(value_node) = self.value.take() {
            seed.deserialize(ValueNodeDeserializer::new(value_node))
        } else {
            Err(Error::Custom(format!("no value for key: {:?}", self.key)))
        }
    }
}

// 列挙型アクセス用のヘルパー構造体
struct TreeEnumAccess {
    variant: String,
    value: Option<ValueNode>,
}

impl TreeEnumAccess {
    fn new(variant: String, value: Option<ValueNode>) -> Self {
        TreeEnumAccess { variant, value }
    }
}

impl<'de> de::EnumAccess<'de> for TreeEnumAccess {
    type Error = Error;
    type Variant = TreeVariantAccess;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant = self.variant.into_deserializer();
        let visitor = TreeVariantAccess { node: self.value };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

// 列挙型バリアントアクセス用のヘルパー構造体
struct TreeVariantAccess {
    node: Option<ValueNode>,
}

impl<'de> de::VariantAccess<'de> for TreeVariantAccess {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        match self.node {
            Some(_) => Err(Error::Custom(
                "invalid type: expected unit variant, found non-unit variant".to_string(),
            )),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.node {
            Some(node) => seed.deserialize(ValueNodeDeserializer::new(node)),
            None => Err(Error::Custom(
                "invalid type: expected newtype variant, found unit variant".to_string(),
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.node {
            Some(node) => match node {
                ValueNode::Array(_) => {
                    let deserializer = ValueNodeDeserializer::new(node);
                    deserializer.deserialize_seq(visitor)
                }
                _ => Err(Error::Custom(
                    "invalid type: expected tuple variant, found non-array".to_string(),
                )),
            },
            None => Err(Error::Custom(
                "invalid type: expected tuple variant, found unit variant".to_string(),
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.node {
            Some(node) => match node {
                ValueNode::Object(_) => {
                    let deserializer = ValueNodeDeserializer::new(node);
                    deserializer.deserialize_map(visitor)
                }
                _ => Err(Error::Custom(
                    "invalid type: expected struct variant, found non-object".to_string(),
                )),
            },
            None => Err(Error::Custom(
                "invalid type: expected struct variant, found unit variant".to_string(),
            )),
        }
    }
}

/// Deserialize an instance of type T from a Tree
pub fn from_value_node<T>(value_node: ValueNode) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    let deserializer = ValueNodeDeserializer::new(value_node);
    T::deserialize(deserializer)
}

/// Deserialize an instance of type T from a Value
pub fn from_node<T>(node: ValueNode) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    let deserializer = ValueNodeDeserializer::new(node);
    T::deserialize(deserializer)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use serde::Deserialize;
    use std::str::FromStr;

    #[test]
    fn test_deserialize_null() {
        let json = "null";
        let value_node = ValueNode::from_str(json).unwrap();
        assert!(value_node.is_null());
    }

    #[test]
    fn test_deserialize_bool() {
        let json = "true";
        let value_node = ValueNode::from_str(json).unwrap();
        assert!(value_node.is_bool());
        pretty_assertions::assert_eq!(value_node.as_bool(), Some(true));

        let json = "false";
        let value_node = ValueNode::from_str(json).unwrap();
        assert!(value_node.is_bool());
        pretty_assertions::assert_eq!(value_node.as_bool(), Some(false));
    }

    #[test]
    fn test_deserialize_number() {
        let json = "42";
        let value_node = ValueNode::from_str(json).unwrap();
        assert!(value_node.is_number());
        pretty_assertions::assert_eq!(value_node.as_f64(), Some(42.0));

        let json = "-3.02";
        let value_node = ValueNode::from_str(json).unwrap();
        assert!(value_node.is_number());
        pretty_assertions::assert_eq!(value_node.as_f64(), Some(-3.02));
    }

    #[test]
    fn test_deserialize_string() {
        let json = r#""hello""#;
        let value_node = ValueNode::from_str(json).unwrap();
        assert!(value_node.is_string());
        pretty_assertions::assert_eq!(value_node.as_str(), Some("hello"));
    }

    #[test]
    fn test_deserialize_array() {
        let json = "[1, 2, 3]";
        let value_node = ValueNode::from_str(json).unwrap();
        assert!(value_node.is_array());

        let json = "[]";
        let value_node = ValueNode::from_str(json).unwrap();
        assert!(value_node.is_array());
    }

    #[test]
    fn test_deserialize_object() {
        let json = r#"{"a": 1, "b": 2}"#;
        let value_node = ValueNode::from_str(json).unwrap();
        assert!(value_node.is_object());

        let json = "{}";
        let value_node = ValueNode::from_str(json).unwrap();
        assert!(value_node.is_object());
    }

    #[test]
    fn test_from_reader() {
        let json = r#"{"name": "John", "age": 30}"#;
        let cursor = Cursor::new(json);

        let node = ValueNode::from_reader(cursor).unwrap();
        assert!(node.is_object());

        if let Some(obj) = node.as_object() {
            pretty_assertions::assert_eq!(obj.len(), 2);

            if let Some(name_node) = obj.properties.get("name") {
                pretty_assertions::assert_eq!(name_node.as_str(), Some("John"));
            } else {
                panic!("name property not found");
            }

            if let Some(age_node) = obj.properties.get("age") {
                pretty_assertions::assert_eq!(age_node.as_i64(), Some(30));
            } else {
                panic!("age property not found");
            }
        }
    }

    #[test]
    fn test_deserialize_complex() {
        let json = r#"
        {
            "name": "John",
            "age": 30,
            "isStudent": false,
            "courses": ["Math", "Physics"],
            "address": {
                "city": "New York",
                "zip": "10001"
            }
        }
        "#;

        let value_node = ValueNode::from_str(json).unwrap();
        assert!(value_node.is_object());

        // Convert to Value for easier testing
        let value: Value = value_node.into();
        let obj = value.as_object().unwrap();
        pretty_assertions::assert_eq!(obj.get("name").unwrap().as_str(), Some("John"));
        pretty_assertions::assert_eq!(obj.get("age").unwrap().as_i64(), Some(30));
        pretty_assertions::assert_eq!(obj.get("isStudent").unwrap().as_bool(), Some(false));

        let courses = obj.get("courses").unwrap().as_array().unwrap();
        pretty_assertions::assert_eq!(courses.len(), 2);
        pretty_assertions::assert_eq!(courses[0].as_str(), Some("Math"));
        pretty_assertions::assert_eq!(courses[1].as_str(), Some("Physics"));

        let address = obj.get("address").unwrap().as_object().unwrap();
        pretty_assertions::assert_eq!(address.get("city").unwrap().as_str(), Some("New York"));
        pretty_assertions::assert_eq!(address.get("zip").unwrap().as_str(), Some("10001"));
    }

    #[test]
    fn test_deserialize_to_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Person {
            name: String,
            age: u8,
            is_student: bool,
        }

        let json = r#"{"name": "John", "age": 30, "is_student": false}"#;
        let person: Person = from_str(json).unwrap();

        pretty_assertions::assert_eq!(person.name, "John");
        pretty_assertions::assert_eq!(person.age, 30);
        pretty_assertions::assert_eq!(person.is_student, false);
    }

    #[test]
    fn test_deserialize_to_nested_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Address {
            city: String,
            zip: String,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        struct Person {
            name: String,
            age: u8,
            address: Address,
        }

        let json = r#"
        {
            "name": "John",
            "age": 30,
            "address": {
                "city": "New York",
                "zip": "10001"
            }
        }
        "#;

        let person: Person = from_str(json).unwrap();

        pretty_assertions::assert_eq!(person.name, "John");
        pretty_assertions::assert_eq!(person.age, 30);
        pretty_assertions::assert_eq!(person.address.city, "New York");
        pretty_assertions::assert_eq!(person.address.zip, "10001");
    }

    #[test]
    fn test_deserialize_to_enum() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Color {
            Red,
            Green,
            Blue,
            Rgb(u8, u8, u8),
            HexCode(String),
        }

        let json = r#""Red""#;
        let color: Color = from_str(json).unwrap();
        pretty_assertions::assert_eq!(color, Color::Red);

        let json = r#"{"Rgb": [255, 255, 255]}"#;
        let color: Color = from_str(json).unwrap();
        pretty_assertions::assert_eq!(color, Color::Rgb(255, 255, 255));

        let json = r###"{"HexCode": "#FFFFFF"}"###;
        let color: Color = from_str(json).unwrap();
        pretty_assertions::assert_eq!(color, Color::HexCode("#FFFFFF".to_string()));
    }

    #[test]
    fn test_source_position() {
        let json = r#"{"name": "John", "age": 30}"#;
        let value_node = ValueNode::from_str(json).unwrap();

        // `tree.root.range`や子要素のrangeを調べることで位置情報が取得できる
        assert!(value_node.range().start != value_node.range().end);

        if let Some(object_node) = value_node.as_object() {
            // オブジェクトのプロパティの位置情報を確認
            if let Some(name_node) = object_node.properties.get("name") {
                // "name"キーの値の位置情報
                assert!(name_node.range().start != name_node.range().end);

                // 値が "John" であることを確認
                pretty_assertions::assert_eq!(name_node.as_str(), Some("John"));
            }

            if let Some(age_node) = object_node.properties.get("age") {
                // "age"キーの値の位置情報
                assert!(age_node.range().start != age_node.range().end);

                // 値が 30 であることを確認
                pretty_assertions::assert_eq!(age_node.as_i64(), Some(30));
            }
        }
    }

    #[test]
    fn test_string_with_quotes() {
        let json = r#""This string contains \"quotes\" inside it""#;
        let value_node = ValueNode::from_str(json).unwrap();
        assert!(value_node.is_string());
        pretty_assertions::assert_eq!(
            value_node.as_str(),
            Some(r#"This string contains "quotes" inside it"#)
        );
    }

    #[test]
    fn test_string_with_escape_sequences() {
        let json = r#""Line1\nLine2\tTabbed\rCarriage Return\\Backslash""#;
        let value_node = ValueNode::from_str(json).unwrap();
        assert!(value_node.is_string());
        pretty_assertions::assert_eq!(
            value_node.as_str(),
            Some("Line1\nLine2\tTabbed\rCarriage Return\\Backslash")
        );
    }

    #[test]
    fn test_unicode_string() {
        let json = r#""こんにちは世界! 🌍 🌎 🌏""#;
        let value_node = ValueNode::from_str(json).unwrap();
        assert!(value_node.is_string());
        pretty_assertions::assert_eq!(value_node.as_str(), Some("こんにちは世界! 🌍 🌎 🌏"));
    }

    #[test]
    fn test_unicode_escape_sequence() {
        let json = r#""\u3053\u3093\u306B\u3061\u306F\u4E16\u754C""#;
        let value_node = ValueNode::from_str(json).unwrap();
        assert!(value_node.is_string());
        pretty_assertions::assert_eq!(value_node.as_str(), Some("こんにちは世界"));
    }

    #[test]
    fn test_complex_string_mixed() {
        let json = r#""Mixed: \"quotes\", 日本語, and \u0065\u0073\u0063\u0061\u0070\u0065\u0064 text with 🚀""#;
        let value_node = ValueNode::from_str(json).unwrap();
        assert!(value_node.is_string());
        pretty_assertions::assert_eq!(
            value_node.as_str(),
            Some(r#"Mixed: "quotes", 日本語, and escaped text with 🚀"#)
        );
    }

    #[test]
    fn test_string_with_control_characters() {
        let json = r#""\u0000\u0001\u0002\b\f""#;
        let value_node = ValueNode::from_str(json).unwrap();
        assert!(value_node.is_string());
        pretty_assertions::assert_eq!(
            value_node.as_str(),
            Some("\u{0000}\u{0001}\u{0002}\u{0008}\u{000C}")
        );
    }

    #[test]
    fn test_surrogate_pairs() {
        // Test emoji using surrogate pairs (🎉 = U+1F389)
        let json = r#""\uD83C\uDF89""#;
        let value_node = ValueNode::from_str(json).unwrap();
        assert!(value_node.is_string());
        pretty_assertions::assert_eq!(value_node.as_str(), Some("🎉"));

        // Test another emoji (😀 = U+1F600)
        let json = r#""\uD83D\uDE00""#;
        let value_node = ValueNode::from_str(json).unwrap();
        assert!(value_node.is_string());
        pretty_assertions::assert_eq!(value_node.as_str(), Some("😀"));

        // Test mathematical alphanumeric symbols (𝐀 = U+1D400)
        let json = r#""\uD835\uDC00""#;
        let value_node = ValueNode::from_str(json).unwrap();
        assert!(value_node.is_string());
        pretty_assertions::assert_eq!(value_node.as_str(), Some("𝐀"));
    }

    #[test]
    fn test_invalid_surrogate_pairs() {
        // High surrogate without low surrogate
        let json = r#""\uD800""#;
        assert!(ValueNode::from_str(json).is_err());

        // Low surrogate without high surrogate
        let json = r#""\uDC00""#;
        assert!(ValueNode::from_str(json).is_err());

        // High surrogate followed by non-surrogate
        let json = r#""\uD800\u0041""#;
        assert!(ValueNode::from_str(json).is_err());
    }

    #[test]
    fn test_mixed_surrogate_and_regular() {
        // Mix of regular characters and surrogate pairs
        let json = r#""Hello \uD83D\uDE00 World!""#;
        let value_node = ValueNode::from_str(json).unwrap();
        assert!(value_node.is_string());
        pretty_assertions::assert_eq!(value_node.as_str(), Some("Hello 😀 World!"));
    }
}
