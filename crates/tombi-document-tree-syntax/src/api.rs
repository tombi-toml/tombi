use tombi_document_tree as api;

macro_rules! impl_node {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl<'t> api::Node for $ty {
                #[inline]
                fn span(&self) -> tombi_text::Span {
                    <$ty>::span(self)
                }

            }
        )+
    };
}

impl_node!(
    crate::Boolean,
    crate::Integer,
    crate::Float,
    crate::String<'t>,
    crate::OffsetDateTime,
    crate::LocalDateTime,
    crate::LocalDate,
    crate::LocalTime,
);

macro_rules! impl_node_with_symbol_span {
    ($($ty:ident),+ $(,)?) => {
        $(
            impl api::Node for crate::$ty<'_> {
                #[inline]
                fn span(&self) -> tombi_text::Span {
                    crate::$ty::span(self)
                }

                #[inline]
                fn symbol_span(&self) -> tombi_text::Span {
                    crate::$ty::symbol_span(self)
                }
            }
        )+
    };
}

impl_node_with_symbol_span!(Array, Table, Value);

impl api::Node for crate::Key<'_> {
    #[inline]
    fn span(&self) -> tombi_text::Span {
        self.span()
    }
}

impl<'t> api::DocumentTree for crate::DocumentTree<'t> {
    type Table = crate::Table<'t>;

    #[inline]
    fn root(&self) -> &Self::Table {
        self
    }
}

impl api::Key for crate::Key<'_> {
    #[inline]
    fn kind(&self) -> api::KeyKind {
        self.kind()
    }

    #[inline]
    fn content(&self) -> &str {
        self.value()
    }

    #[inline]
    fn unquoted_span(&self) -> tombi_text::Span {
        self.unquoted_span()
    }
}

impl<'t> api::Array for crate::Array<'t> {
    type Value = crate::Value<'t>;

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

impl<'t> api::Table for crate::Table<'t> {
    type Key = crate::Key<'t>;
    type Value = crate::Value<'t>;

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

impl<'t> api::ValueNode for crate::Value<'t> {
    type Array = crate::Array<'t>;
    type Table = crate::Table<'t>;

    #[inline]
    fn value(&self) -> api::Value<'_, crate::Array<'t>, crate::Table<'t>> {
        match self {
            crate::Value::Boolean(value) => {
                api::Value::Boolean(api::BooleanValue::new(value.value(), value.span()))
            }
            crate::Value::Integer(value) => api::Value::Integer(api::IntegerValue::new(
                value.kind(),
                value.value(),
                value.span(),
            )),
            crate::Value::Float(value) => {
                api::Value::Float(api::FloatValue::new(value.value(), value.span()))
            }
            crate::Value::String(value) => api::Value::String(api::StringValue::new(
                value.kind(),
                value.value(),
                value.span(),
            )),
            crate::Value::OffsetDateTime(value) => api::Value::OffsetDateTime(
                api::OffsetDateTimeValue::new(value.value(), value.span()),
            ),
            crate::Value::LocalDateTime(value) => {
                api::Value::LocalDateTime(api::LocalDateTimeValue::new(value.value(), value.span()))
            }
            crate::Value::LocalDate(value) => {
                api::Value::LocalDate(api::LocalDateValue::new(value.value(), value.span()))
            }
            crate::Value::LocalTime(value) => {
                api::Value::LocalTime(api::LocalTimeValue::new(value.value(), value.span()))
            }
            crate::Value::Array(value) => api::Value::Array(value),
            crate::Value::Table(value) => api::Value::Table(value),
            crate::Value::Incomplete { span } => api::Value::Incomplete { span: *span },
        }
    }
}
