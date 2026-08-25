//! `tombi-ast` contract implemented by the source-backed syntax tape.

use std::str::FromStr;

use crate::{AstNode as _, AstToken as _};

macro_rules! impl_node {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl tombi_ast::Node for $ty {
                #[inline]
                fn range(&self) -> tombi_text::Range {
                    self.syntax().range()
                }

                #[inline]
                fn text(&self) -> &str {
                    self.syntax().text()
                }
            }
        )+
    };
}

impl_node!(
    crate::Root,
    crate::RootItem,
    crate::Table,
    crate::ArrayOfTable,
    crate::KeyValue,
    crate::Keys,
    crate::Key,
    crate::Value,
    crate::Array,
    crate::InlineTable,
);

impl tombi_ast::Node for crate::Comment {
    #[inline]
    fn range(&self) -> tombi_text::Range {
        self.syntax().range()
    }

    #[inline]
    fn text(&self) -> &str {
        self.syntax().text()
    }
}

impl tombi_ast::CommentNode for crate::Comment {}

impl tombi_ast::RootNode for crate::Root {
    type Item = crate::RootItem;
    type KeyValue = crate::KeyValue;
    type Table = crate::Table;
    type ArrayOfTable = crate::ArrayOfTable;

    #[inline]
    fn items(&self) -> impl Iterator<Item = Self::Item> + '_ {
        crate::Root::items(self)
    }

    #[inline]
    fn key_values(&self) -> impl Iterator<Item = Self::KeyValue> + '_ {
        crate::Root::key_values(self)
    }

    #[inline]
    fn tables(&self) -> impl Iterator<Item = Self::Table> + '_ {
        crate::Root::items(self).filter_map(|item| match item {
            crate::RootItem::Table(table) => Some(table),
            _ => None,
        })
    }

    #[inline]
    fn array_of_tables(&self) -> impl Iterator<Item = Self::ArrayOfTable> + '_ {
        crate::Root::items(self).filter_map(|item| match item {
            crate::RootItem::ArrayOfTable(array_of_table) => Some(array_of_table),
            _ => None,
        })
    }
}

impl tombi_ast::RootItemNode for crate::RootItem {
    type KeyValue = crate::KeyValue;
    type Table = crate::Table;
    type ArrayOfTable = crate::ArrayOfTable;

    #[inline]
    fn item(&self) -> tombi_ast::RootItem<'_, crate::KeyValue, crate::Table, crate::ArrayOfTable> {
        match self {
            Self::KeyValue(key_value) => tombi_ast::RootItem::KeyValue(key_value),
            Self::Table(table) => tombi_ast::RootItem::Table(table),
            Self::ArrayOfTable(array_of_table) => tombi_ast::RootItem::ArrayOfTable(array_of_table),
        }
    }
}

macro_rules! impl_table {
    ($trait:ident for $ty:ty) => {
        impl tombi_ast::$trait for $ty {
            type Keys = crate::Keys;
            type KeyValue = crate::KeyValue;

            #[inline]
            fn header(&self) -> Option<Self::Keys> {
                <$ty>::header(self)
            }

            #[inline]
            fn key_values(&self) -> impl Iterator<Item = Self::KeyValue> + '_ {
                <$ty>::key_values(self)
            }
        }
    };
}

impl_table!(TableNode for crate::Table);
impl_table!(ArrayOfTableNode for crate::ArrayOfTable);

impl tombi_ast::KeyValueNode for crate::KeyValue {
    type Keys = crate::Keys;
    type Value = crate::Value;

    #[inline]
    fn keys(&self) -> Option<Self::Keys> {
        crate::KeyValue::keys(self)
    }

    #[inline]
    fn value(&self) -> Option<Self::Value> {
        crate::KeyValue::value(self)
    }
}

impl tombi_ast::KeysNode for crate::Keys {
    type Key = crate::Key;

    #[inline]
    fn keys(&self) -> impl Iterator<Item = Self::Key> + '_ {
        crate::Keys::keys(self)
    }
}

impl tombi_ast::KeyNode for crate::Key {
    #[inline]
    fn content(
        &self,
        toml_version: tombi_toml_version::TomlVersion,
    ) -> Option<std::borrow::Cow<'_, str>> {
        self.try_to_content(toml_version).ok()
    }
}

impl tombi_ast::ValueNode for crate::Value {
    type Array = crate::Array;
    type InlineTable = crate::InlineTable;

    #[inline]
    fn value(
        &self,
        toml_version: tombi_toml_version::TomlVersion,
    ) -> Option<
        tombi_ast::Value<
            '_,
            <Self as tombi_ast::ValueNode>::Array,
            <Self as tombi_ast::ValueNode>::InlineTable,
        >,
    > {
        use tombi_ast::Value;

        macro_rules! decode_token {
            ($node:expr, $decode:expr, $variant:ident) => {{
                let value = $decode($node.syntax().text()).ok()?;
                Some(Value::$variant(value))
            }};
        }

        macro_rules! decode_text {
            ($node:expr) => {{
                Some(Value::String(
                    $node.syntax().try_to_content(toml_version).ok()?,
                ))
            }};
        }

        match self {
            Self::Boolean(node) => decode_token!(
                node,
                crate::support::literal::boolean::try_from_boolean,
                Boolean
            ),
            Self::IntegerBin(node) => decode_token!(
                node,
                crate::support::literal::integer::try_from_binary,
                Integer
            ),
            Self::IntegerOct(node) => decode_token!(
                node,
                crate::support::literal::integer::try_from_octal,
                Integer
            ),
            Self::IntegerDec(node) => decode_token!(
                node,
                crate::support::literal::integer::try_from_decimal,
                Integer
            ),
            Self::IntegerHex(node) => decode_token!(
                node,
                crate::support::literal::integer::try_from_hexadecimal,
                Integer
            ),
            Self::Float(node) => {
                decode_token!(node, crate::support::literal::float::try_from_float, Float)
            }
            Self::BasicString(node) => decode_text!(node),
            Self::LiteralString(node) => decode_text!(node),
            Self::MultiLineBasicString(node) => decode_text!(node),
            Self::MultiLineLiteralString(node) => decode_text!(node),
            Self::OffsetDateTime(node) => {
                let text = normalized_date_time_text(node, toml_version)?;
                tombi_ast::OffsetDateTime::from_str(&text)
                    .map(Value::OffsetDateTime)
                    .ok()
            }
            Self::LocalDateTime(node) => {
                let text = normalized_date_time_text(node, toml_version)?;
                tombi_ast::LocalDateTime::from_str(&text)
                    .map(Value::LocalDateTime)
                    .ok()
            }
            Self::LocalDate(node) => decode_token!(node, tombi_ast::LocalDate::from_str, LocalDate),
            Self::LocalTime(node) => {
                let text = node.syntax().text();
                if toml_version == tombi_toml_version::TomlVersion::V1_0_0
                    && text.chars().nth("00:00".len()) != Some(':')
                {
                    return None;
                }
                tombi_ast::LocalTime::from_str(text)
                    .map(Value::LocalTime)
                    .ok()
            }
            Self::Array(array) => Some(Value::Array(array)),
            Self::InlineTable(table) => Some(Value::InlineTable(table)),
        }
    }
}

fn normalized_date_time_text(
    node: &impl crate::AstNode,
    toml_version: tombi_toml_version::TomlVersion,
) -> Option<String> {
    const DEFAULT_SECONDS: &str = ":00";
    const DATE_SIZE: usize = "2024-12-31".len();
    const DATE_TIME_WITHOUT_SECONDS_SIZE: usize = "2024-01-01T00:00".len();

    let source = node.syntax().text();
    let mut decoded = String::with_capacity(source.len() + DEFAULT_SECONDS.len());

    for (index, character) in source.char_indices() {
        if index == DATE_SIZE && matches!(character, 'T' | 't') {
            decoded.push(' ');
        } else if index == DATE_TIME_WITHOUT_SECONDS_SIZE && character != ':' {
            if toml_version == tombi_toml_version::TomlVersion::V1_0_0 {
                return None;
            }
            decoded.push_str(DEFAULT_SECONDS);
            decoded.push(character);
        } else {
            decoded.push(character);
        }
    }

    if decoded.len() == DATE_TIME_WITHOUT_SECONDS_SIZE {
        if toml_version == tombi_toml_version::TomlVersion::V1_0_0 {
            return None;
        }
        decoded.push_str(DEFAULT_SECONDS);
    }

    Some(decoded)
}

impl tombi_ast::ArrayNode for crate::Array {
    type Value = crate::Value;

    #[inline]
    fn values(&self) -> impl Iterator<Item = Self::Value> + '_ {
        crate::Array::values(self)
    }
}

impl tombi_ast::InlineTableNode for crate::InlineTable {
    type KeyValue = crate::KeyValue;

    #[inline]
    fn key_values(&self) -> impl Iterator<Item = Self::KeyValue> + '_ {
        crate::InlineTable::key_values(self)
    }
}
