#[derive(Debug, Clone, PartialEq)]
pub struct Float {
    value: f64,
}

impl Float {
    #[inline]
    pub fn new(value: f64) -> Self {
        Self { value }
    }

    #[inline]
    pub fn value(&self) -> f64 {
        self.value
    }
}

impl From<tombi_document_tree_syntax::Float> for Float {
    fn from(node: tombi_document_tree_syntax::Float) -> Self {
        Self {
            value: node.value(),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Float {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Float {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        f64::deserialize(deserializer).map(|value| Self { value })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl Float {
        fn try_new(value: &str) -> Result<Self, Box<dyn std::error::Error>> {
            Ok(Self {
                value: value.parse()?,
            })
        }
    }

    #[test]
    fn inf() {
        let float = crate::value::Float::try_new("inf").unwrap();
        pretty_assertions::assert_eq!(float.value(), f64::INFINITY);
    }

    #[test]
    fn p_inf() {
        let float = crate::value::Float::try_new("+inf").unwrap();
        pretty_assertions::assert_eq!(float.value(), f64::INFINITY);
    }

    #[test]
    fn m_inf() {
        let float = crate::value::Float::try_new("-inf").unwrap();
        pretty_assertions::assert_eq!(float.value(), f64::NEG_INFINITY);
    }

    #[test]
    fn nan() {
        let float = crate::value::Float::try_new("nan").unwrap();
        assert!(float.value().is_nan());
    }

    #[test]
    fn p_nan() {
        let float = crate::value::Float::try_new("+nan").unwrap();
        assert!(float.value().is_nan());
    }

    #[test]
    fn m_nan() {
        let float = crate::value::Float::try_new("-nan").unwrap();
        assert!(float.value().is_nan());
    }
}
