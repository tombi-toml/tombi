use tombi_document_tree as api;

macro_rules! impl_node {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl api::Node for $ty {
                #[inline]
                fn range(&self) -> tombi_text::Range {
                    <$ty>::range(self)
                }

            }
        )+
    };
}

impl_node!(
    crate::Boolean,
    crate::Integer,
    crate::Float,
    crate::String,
    crate::OffsetDateTime,
    crate::LocalDateTime,
    crate::LocalDate,
    crate::LocalTime,
);

macro_rules! impl_node_with_symbol_range {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl api::Node for $ty {
                #[inline]
                fn range(&self) -> tombi_text::Range {
                    <$ty>::range(self)
                }

                #[inline]
                fn symbol_range(&self) -> tombi_text::Range {
                    <$ty>::symbol_range(self)
                }
            }
        )+
    };
}

impl_node_with_symbol_range!(crate::Array, crate::Table, crate::Value);

impl api::Node for crate::Key {
    #[inline]
    fn range(&self) -> tombi_text::Range {
        self.range()
    }
}

impl api::DocumentTree for crate::DocumentTree {
    type Table = crate::Table;

    #[inline]
    fn root(&self) -> &Self::Table {
        self
    }
}

impl api::Key for crate::Key {
    #[inline]
    fn kind(&self) -> api::KeyKind {
        self.kind()
    }

    #[inline]
    fn content(&self) -> &str {
        self.value()
    }

    #[inline]
    fn unquoted_range(&self) -> tombi_text::Range {
        self.unquoted_range()
    }
}

impl api::Array for crate::Array {
    type Value = crate::Value;

    #[inline]
    fn kind(&self) -> api::ArrayKind {
        self.kind()
    }

    #[inline]
    fn get(&self, index: usize) -> Option<&Self::Value> {
        self.get(index)
    }

    #[inline]
    fn values(&self) -> impl Iterator<Item = &Self::Value> + '_ {
        self.iter()
    }
}

impl api::Table for crate::Table {
    type Key = crate::Key;
    type Value = crate::Value;

    #[inline]
    fn kind(&self) -> api::TableKind {
        self.kind()
    }

    #[inline]
    fn get(&self, key: &str) -> Option<&Self::Value> {
        self.get(key)
    }

    #[inline]
    fn get_key_value(&self, key: &str) -> Option<(&Self::Key, &Self::Value)> {
        self.get_key_value(key)
    }

    #[inline]
    fn entries(&self) -> impl Iterator<Item = (&Self::Key, &Self::Value)> + '_ {
        self.key_values().iter()
    }
}

impl api::ValueNode for crate::Value {
    type Array = crate::Array;
    type Table = crate::Table;

    #[inline]
    fn value(&self) -> api::Value<'_, crate::Array, crate::Table> {
        match self {
            crate::Value::Boolean(value) => {
                api::Value::Boolean(api::BooleanValue::new(value.value(), value.range()))
            }
            crate::Value::Integer(value) => api::Value::Integer(api::IntegerValue::new(
                value.kind(),
                value.value(),
                value.range(),
            )),
            crate::Value::Float(value) => {
                api::Value::Float(api::FloatValue::new(value.value(), value.range()))
            }
            crate::Value::String(value) => api::Value::String(api::StringValue::new(
                value.kind(),
                value.value(),
                value.range(),
            )),
            crate::Value::OffsetDateTime(value) => api::Value::OffsetDateTime(
                api::OffsetDateTimeValue::new(value.value(), value.range()),
            ),
            crate::Value::LocalDateTime(value) => api::Value::LocalDateTime(
                api::LocalDateTimeValue::new(value.value(), value.range()),
            ),
            crate::Value::LocalDate(value) => {
                api::Value::LocalDate(api::LocalDateValue::new(value.value(), value.range()))
            }
            crate::Value::LocalTime(value) => {
                api::Value::LocalTime(api::LocalTimeValue::new(value.value(), value.range()))
            }
            crate::Value::Array(value) => api::Value::Array(value),
            crate::Value::Table(value) => api::Value::Table(value),
            crate::Value::Incomplete { range } => api::Value::Incomplete { range: *range },
        }
    }
}
