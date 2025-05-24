use indexmap::{map::Entry, IndexMap};
use serde::forward_to_deserialize_any;

use crate::{IntoDocument, Key, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableKind {
    Table,
    InlineTable,
    KeyValue,
}

impl From<tombi_document_tree::TableKind> for TableKind {
    fn from(kind: tombi_document_tree::TableKind) -> Self {
        use tombi_document_tree::TableKind::*;
        match kind {
            Root | Table | ParentTable | ParentKey => Self::Table,
            InlineTable => Self::InlineTable,
            KeyValue => Self::KeyValue,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    kind: TableKind,
    key_values: IndexMap<Key, Value>,
}

impl Table {
    pub fn new(kind: TableKind) -> Self {
        Self {
            kind,
            key_values: IndexMap::new(),
        }
    }

    #[inline]
    pub fn kind(&self) -> TableKind {
        self.kind
    }

    #[inline]
    pub fn kind_mut(&mut self) -> &mut TableKind {
        &mut self.kind
    }

    #[inline]
    pub fn key_values(&self) -> &IndexMap<Key, Value> {
        &self.key_values
    }

    pub fn entry(&mut self, key: Key) -> Entry<'_, Key, Value> {
        self.key_values.entry(key)
    }

    pub fn insert(&mut self, key: Key, value: Value) {
        self.key_values.insert(key, value);
    }
}

impl IntoDocument<Table> for tombi_document_tree::Table {
    fn into_document(self, toml_version: crate::TomlVersion) -> Table {
        let kind = self.kind().into();
        let key_values = IndexMap::<tombi_document_tree::Key, tombi_document_tree::Value>::from(self)
            .into_iter()
            .map(|(key, value)| {
                (
                    key.into_document(toml_version),
                    value.into_document(toml_version),
                )
            })
            .collect();

        Table { kind, key_values }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Table {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.key_values.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Table {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let key_values = IndexMap::<Key, Value>::deserialize(deserializer)?;
        Ok(Self {
            kind: TableKind::Table,
            key_values,
        })
    }
}

#[cfg(feature = "serde")]
struct TableDeserializer<'de> {
    iter: <&'de IndexMap<Key, Value> as IntoIterator>::IntoIter,
    value: Option<&'de Value>,
}

#[cfg(feature = "serde")]
impl<'de> TableDeserializer<'de> {
    fn new(table: &'de Table) -> Self {
        Self {
            iter: table.key_values().iter(),
            value: None,
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::de::MapAccess<'de> for TableDeserializer<'de> {
    type Error = crate::de::Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(key).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::custom("value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserializer<'de> for &'de Table {
    type Error = crate::de::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_map(TableDeserializer::new(self))
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
        let mut iter = self.key_values().iter();
        let (key, value) = match iter.next() {
            Some(v) => v,
            None => {
                return Err(serde::de::Error::invalid_value(
                    serde::de::Unexpected::Map,
                    &"map with a single key",
                ));
            }
        };
        // enums are encoded in json as maps with a single key:value pair
        if iter.next().is_some() {
            return Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Map,
                &"map with a single key",
            ));
        }

        visitor.visit_enum(super::EnumRefDeserializer {
            variant: key.value(),
            value: Some(value),
        })
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct struct identifier ignored_any
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::test_deserialize;

    test_deserialize!(
        #[test]
        fn key_value(
            r#"
            key = "value"
            "#
        ) -> Ok(json!({"key": "value"}))
    );

    test_deserialize!(
        #[test]
        fn keys_value(
            r#"
            foo.bar.baz = "value"
            "#
        ) -> Ok(json!(
            {
                "foo": {
                    "bar": {
                        "baz": "value"
                    }
                }
            }
        ))
    );

    test_deserialize!(
        #[test]
        fn table(
            r#"
            [foo]
            bar = "value"
            baz = 42
            "#
        ) -> Ok(json!({"foo": {"bar": "value", "baz": 42}}))
    );

    test_deserialize! {
        #[test]
        fn tables(
            r#"
            [foo1]
            bar = 1

            [foo2]
            bar = 2
            "#
        ) -> Ok(json!({
            "foo1": {"bar": 1},
            "foo2": {"bar": 2}
        }))
    }

    test_deserialize! {
        #[test]
        fn sub_empty(
            r#"
            [a]
            [a.b]
            "#
        ) -> Ok(json!({ "a": { "b": {} } }))
    }

    test_deserialize! {
        #[test]
        fn key_dotted_2(
            r#"many.dots.here.dot.dot.dot = {a.b.c = 1, a.b.d = 2}"#
        ) -> Ok(json!(
            {
                "many": {
                    "dots": {
                        "here": {
                            "dot": {
                                "dot": {
                                    "dot": {
                                        "a": {
                                            "b": {
                                                "c": 1,
                                                "d": 2
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        ))
    }

    test_deserialize! {
        #[test]
        fn key_dotted_3(
            r#"
            [tbl]
            a.b.c = {d.e=1}

            [tbl.x]
            a.b.c = {d.e=1}
            "#
        ) -> Ok(json!(
            {
                "tbl": {
                    "a": { "b": { "c": { "d": { "e": 1 } } } },
                    "x": { "a": { "b": { "c": { "d": { "e": 1 } } } } }
                }
            }
        ))
    }

    test_deserialize! {
        #[test]
        fn array_of_table_table(
            r#"
            [[aaa]]
            key = "value"

            [aaa.bbb]
            ccc = true
            "#
        ) -> Ok(
            json!({
                "aaa": [
                    {
                        "key": "value",
                        "bbb": {
                            "ccc": true
                        }
                    }
                ]
            })
        )
    }

    test_deserialize! {
        #[test]
        fn array_of_table_table_twice(
            r#"
            [[aaa]]
            key = "value"

            [aaa.bbb]
            ccc = true

            [[aaa]]
            [aaa.bbb]
            ccc = false
            "#
        ) -> Ok(
            json!({
                "aaa": [
                    {
                        "key": "value",
                        "bbb": {
                            "ccc": true
                        }
                    },
                    {
                        "bbb": {
                            "ccc": false
                        }
                    }
                ]
            })
        )
    }

    test_deserialize! {
        #[test]
        fn duplicate(
            r#"
            [a]
            b = 1

            [a]
            c = 2
            "#
        ) -> Err([
            ("conflicting table", ((3, 0), (4, 5)))
        ])
    }

    test_deserialize! {
        #[test]
        fn duplicate_key_2(
            r#"
            a.b=0
            # Since table "a" is already defined, it can't be replaced by an inline table.
            a={}
       "#
        ) -> Err([
            ("conflicting table", ((2, 2), (2, 4)))
        ])
    }

    test_deserialize! {
        #[test]
        fn duplicate_key_3(
            r#"tbl = { fruit = { apple.color = "red" }, fruit.apple.texture = { smooth = true } }"#
        ) -> Err([
            ("conflicting table", ((0, 41), (0, 80)))
        ])
    }

    test_deserialize! {
        #[test]
        fn duplicate_key_dotted_table2(
            r#"
            [fruit]
            apple.taste.sweet = true

            [fruit.apple.taste] # INVALID
            "#
        ) -> Err([
            ("conflicting table", ((3, 0), (3, 29)))
        ])
    }

    test_deserialize! {
        #[test]
        fn redefine_2(
            r#"
            [t1]
            t2.t3.v = 0
            [t1.t2]
            "#
        ) -> Err([
            ("conflicting table", ((2, 0), (2, 7)))
        ])
    }
}
