use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub enum LiteralValueRef<'a> {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(&'a str),
    OffsetDateTime(&'a tombi_date_time::OffsetDateTime),
    LocalDateTime(&'a tombi_date_time::LocalDateTime),
    LocalDate(&'a tombi_date_time::LocalDate),
    LocalTime(&'a tombi_date_time::LocalTime),
}

impl<'a> std::fmt::Display for LiteralValueRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use LiteralValueRef::*;

        match self {
            Boolean(v) => v.fmt(f),
            Integer(v) => v.fmt(f),
            Float(v) => v.fmt(f),
            String(v) => write!(f, "\"{v}\""),
            OffsetDateTime(v) => v.fmt(f),
            LocalDateTime(v) => v.fmt(f),
            LocalDate(v) => v.fmt(f),
            LocalTime(v) => v.fmt(f),
        }
    }
}

impl<'a> PartialEq for LiteralValueRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        use LiteralValueRef::*;
        match (self, other) {
            (Boolean(a), Boolean(b)) => a == b,
            (Integer(a), Integer(b)) => a == b,
            (Float(a), Float(b)) => a.total_cmp(b) == Ordering::Equal,
            (String(a), String(b)) => a == b,
            (OffsetDateTime(a), OffsetDateTime(b)) => a == b,
            (LocalDateTime(a), LocalDateTime(b)) => a == b,
            (LocalDate(a), LocalDate(b)) => a == b,
            (LocalTime(a), LocalTime(b)) => a == b,
            _ => false,
        }
    }
}

impl<'a> Eq for LiteralValueRef<'a> {}

impl<'a> std::hash::Hash for LiteralValueRef<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use LiteralValueRef::*;

        match self {
            Boolean(v) => v.hash(state),
            Integer(v) => v.hash(state),
            Float(v) => v.to_bits().hash(state),
            String(v) => v.hash(state),
            OffsetDateTime(v) => v.hash(state),
            LocalDateTime(v) => v.hash(state),
            LocalDate(v) => v.hash(state),
            LocalTime(v) => v.hash(state),
        }
    }
}

impl<'a> From<&'a crate::Value> for Option<LiteralValueRef<'a>> {
    fn from(value: &'a crate::Value) -> Self {
        match value {
            crate::Value::Boolean(boolean) => Some(LiteralValueRef::Boolean(boolean.value())),
            crate::Value::Integer(integer) => Some(LiteralValueRef::Integer(integer.value())),
            crate::Value::Float(float) => Some(LiteralValueRef::Float(float.value())),
            crate::Value::String(string) => Some(LiteralValueRef::String(string.value())),
            crate::Value::OffsetDateTime(date_time) => {
                Some(LiteralValueRef::OffsetDateTime(date_time.value()))
            }
            crate::Value::LocalDateTime(date_time) => {
                Some(LiteralValueRef::LocalDateTime(date_time.value()))
            }
            crate::Value::LocalDate(date) => Some(LiteralValueRef::LocalDate(date.value())),
            crate::Value::LocalTime(time) => Some(LiteralValueRef::LocalTime(time.value())),
            _ => None,
        }
    }
}
