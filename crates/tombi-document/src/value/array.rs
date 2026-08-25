use serde::forward_to_deserialize_any;

use crate::{IntoDocument, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArrayKind {
    #[default]
    /// An array of tables.
    ///
    /// ```toml
    /// [[array]]
    /// ```
    ArrayOfTable,

    /// An array.
    ///
    /// ```toml
    /// key = [1, 2, 3]
    /// ```
    Array,
}

impl From<tombi_document_tree::ArrayKind> for ArrayKind {
    fn from(kind: tombi_document_tree::ArrayKind) -> Self {
        use tombi_document_tree::ArrayKind::*;

        match kind {
            ArrayOfTable | ParentArrayOfTable => Self::ArrayOfTable,
            Array => Self::Array,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Array {
    kind: ArrayKind,
    values: Vec<Value>,
}

impl Array {
    pub fn new(kind: ArrayKind) -> Self {
        Self {
            kind,
            values: Vec::new(),
        }
    }

    pub fn push(&mut self, value: Value) {
        self.values.push(value);
    }

    pub fn kind(&self) -> ArrayKind {
        self.kind
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn kind_mut(&mut self) -> &mut ArrayKind {
        &mut self.kind
    }

    pub fn values(&self) -> &[Value] {
        &self.values
    }

    pub fn values_mut(&mut self) -> &mut Vec<Value> {
        &mut self.values
    }
}

impl From<Array> for Vec<Value> {
    fn from(val: Array) -> Self {
        val.values
    }
}

impl IntoDocument<Array> for tombi_document_tree_syntax::Array {
    fn into_document(self, toml_version: tombi_toml_version::TomlVersion) -> Array {
        Array {
            kind: self.kind().into(),
            values: Vec::<tombi_document_tree_syntax::Value>::from(self.values())
                .into_iter()
                .map(|value| value.into_document(toml_version))
                .collect(),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Array {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.values.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Array {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let values = Vec::<Value>::deserialize(deserializer)?;
        Ok(Self {
            kind: ArrayKind::ArrayOfTable,
            values,
        })
    }
}

#[cfg(feature = "serde")]
struct ArrayDeserializer<'de> {
    iter: std::slice::Iter<'de, Value>,
}

#[cfg(feature = "serde")]
impl<'de> ArrayDeserializer<'de> {
    fn new(array: &'de Array) -> Self {
        ArrayDeserializer {
            iter: array.values().iter(),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::de::SeqAccess<'de> for ArrayDeserializer<'de> {
    type Error = crate::de::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::de::Deserializer<'de> for &'de Array {
    type Error = crate::de::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(ArrayDeserializer::new(self))
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct tuple
        tuple_struct map struct identifier enum ignored_any
    }
}
