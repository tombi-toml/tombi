use std::cmp::Ordering;

use crate::support::literal::{
    boolean::try_from_boolean,
    float::try_from_float,
    integer::{try_from_binary, try_from_decimal, try_from_hexadecimal, try_from_octal},
};

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    OffsetDateTime(String),
    LocalDateTime(String),
    LocalDate(String),
    LocalTime(String),
}

impl std::fmt::Display for LiteralValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use LiteralValue::*;

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

impl PartialEq for LiteralValue {
    fn eq(&self, other: &Self) -> bool {
        use LiteralValue::*;
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

impl Eq for LiteralValue {}

impl std::hash::Hash for LiteralValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use LiteralValue::*;

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

impl From<crate::Value> for Option<LiteralValue> {
    fn from(value: crate::Value) -> Self {
        match value {
            crate::Value::Boolean(boolean) => boolean.into(),
            crate::Value::IntegerBin(integer) => integer.into(),
            crate::Value::Float(float) => float.into(),
            crate::Value::BasicString(string) => string.into(),
            crate::Value::OffsetDateTime(date_time) => date_time.into(),
            crate::Value::LocalDateTime(date_time) => date_time.into(),
            crate::Value::LocalDate(date) => date.into(),
            crate::Value::LocalTime(time) => time.into(),
            _ => None,
        }
    }
}

impl From<crate::Boolean> for Option<LiteralValue> {
    fn from(value: crate::Boolean) -> Self {
        let Some(token) = value.token() else {
            return None;
        };

        try_from_boolean(&token.text())
            .ok()
            .map(LiteralValue::Boolean)
    }
}

impl From<crate::IntegerBin> for Option<LiteralValue> {
    fn from(value: crate::IntegerBin) -> Self {
        let Some(token) = value.token() else {
            return None;
        };

        try_from_binary(&token.text())
            .ok()
            .map(LiteralValue::Integer)
    }
}

impl From<crate::IntegerDec> for Option<LiteralValue> {
    fn from(value: crate::IntegerDec) -> Self {
        let Some(token) = value.token() else {
            return None;
        };

        try_from_decimal(&token.text())
            .ok()
            .map(LiteralValue::Integer)
    }
}

impl From<crate::IntegerOct> for Option<LiteralValue> {
    fn from(value: crate::IntegerOct) -> Self {
        let Some(token) = value.token() else {
            return None;
        };

        try_from_octal(&token.text())
            .ok()
            .map(LiteralValue::Integer)
    }
}

impl From<crate::IntegerHex> for Option<LiteralValue> {
    fn from(value: crate::IntegerHex) -> Self {
        let Some(token) = value.token() else {
            return None;
        };

        try_from_hexadecimal(&token.text())
            .ok()
            .map(LiteralValue::Integer)
    }
}

impl From<crate::Float> for Option<LiteralValue> {
    fn from(value: crate::Float) -> Self {
        let Some(token) = value.token() else {
            return None;
        };

        try_from_float(&token.text()).ok().map(LiteralValue::Float)
    }
}

impl From<crate::BasicString> for Option<LiteralValue> {
    fn from(value: crate::BasicString) -> Self {
        let Some(token) = value.token() else {
            return None;
        };

        Some(LiteralValue::String(token.text().to_string()))
    }
}

impl From<crate::OffsetDateTime> for Option<LiteralValue> {
    fn from(value: crate::OffsetDateTime) -> Self {
        let Some(token) = value.token() else {
            return None;
        };

        Some(LiteralValue::OffsetDateTime(token.text().to_string()))
    }
}

impl From<crate::LocalDateTime> for Option<LiteralValue> {
    fn from(value: crate::LocalDateTime) -> Self {
        let Some(token) = value.token() else {
            return None;
        };

        Some(LiteralValue::LocalDateTime(token.text().to_string()))
    }
}

impl From<crate::LocalDate> for Option<LiteralValue> {
    fn from(value: crate::LocalDate) -> Self {
        let Some(token) = value.token() else {
            return None;
        };

        Some(LiteralValue::LocalDate(token.text().to_string()))
    }
}

impl From<crate::LocalTime> for Option<LiteralValue> {
    fn from(value: crate::LocalTime) -> Self {
        let Some(token) = value.token() else {
            return None;
        };

        Some(LiteralValue::LocalTime(token.text().to_string()))
    }
}
