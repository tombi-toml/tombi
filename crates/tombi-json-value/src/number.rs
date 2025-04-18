use std::fmt;

/// Number type that can represent both integers and floating point values
#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    /// Integer value
    Integer(i64),
    /// Floating point value
    Float(f64),
}

impl Number {
    /// Creates a Number from a i64 value
    pub fn from_i64(value: i64) -> Self {
        Number::Integer(value)
    }

    /// Creates a Number from a f64 value
    pub fn from_f64(value: f64) -> Self {
        // Convert whole numbers to integers if possible
        if value.fract() == 0.0 && value >= i64::MIN as f64 && value <= i64::MAX as f64 {
            Number::Integer(value as i64)
        } else {
            Number::Float(value)
        }
    }

    /// Check if the number is an integer
    pub fn is_i64(&self) -> bool {
        matches!(self, Number::Integer(_))
    }

    /// Check if the number is a floating point
    pub fn is_f64(&self) -> bool {
        matches!(self, Number::Float(_))
    }

    /// Get as i64 value if possible
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Number::Integer(i) => Some(*i),
            Number::Float(f) => {
                // If the float is a whole number that fits in an i64, return it
                if f.fract() == 0.0 && *f >= i64::MIN as f64 && *f <= i64::MAX as f64 {
                    Some(*f as i64)
                } else {
                    None
                }
            }
        }
    }

    /// Get as f64 value
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Number::Integer(i) => Some(*i as f64),
            Number::Float(f) => Some(*f),
        }
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Number::Integer(i) => write!(f, "{}", i),
            Number::Float(v) => {
                // Ensure that whole number floats are displayed with .0
                if v.fract() == 0.0 {
                    write!(f, "{}.0", v)
                } else {
                    write!(f, "{}", v)
                }
            }
        }
    }
}

impl From<i8> for Number {
    fn from(i: i8) -> Self {
        Number::Integer(i as i64)
    }
}

impl From<i16> for Number {
    fn from(i: i16) -> Self {
        Number::Integer(i as i64)
    }
}

impl From<i32> for Number {
    fn from(i: i32) -> Self {
        Number::Integer(i as i64)
    }
}

impl From<i64> for Number {
    fn from(i: i64) -> Self {
        Number::Integer(i)
    }
}

impl From<u8> for Number {
    fn from(u: u8) -> Self {
        Number::Integer(u as i64)
    }
}

impl From<u16> for Number {
    fn from(u: u16) -> Self {
        Number::Integer(u as i64)
    }
}

impl From<u32> for Number {
    fn from(u: u32) -> Self {
        Number::Integer(u as i64)
    }
}

impl From<u64> for Number {
    fn from(u: u64) -> Self {
        if u <= i64::MAX as u64 {
            Number::Integer(u as i64)
        } else {
            Number::Float(u as f64)
        }
    }
}

impl From<f32> for Number {
    fn from(f: f32) -> Self {
        Number::from_f64(f as f64)
    }
}

impl From<f64> for Number {
    fn from(f: f64) -> Self {
        Number::from_f64(f)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Number::Integer(i) => serializer.serialize_i64(*i),
            Number::Float(f) => serializer.serialize_f64(*f),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Number {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct NumberVisitor;

        impl<'de> serde::de::Visitor<'de> for NumberVisitor {
            type Value = Number;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a JSON number")
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
                Ok(Number::Integer(value))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
                if value <= i64::MAX as u64 {
                    Ok(Number::Integer(value as i64))
                } else {
                    Ok(Number::Float(value as f64))
                }
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E> {
                Ok(Number::from_f64(value))
            }
        }

        deserializer.deserialize_any(NumberVisitor)
    }
}
