pub use tombi_document_tree::IntegerKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Integer {
    kind: IntegerKind,
    value: i64,
}

impl Integer {
    #[inline]
    pub fn new(value: i64) -> Self {
        Self {
            kind: IntegerKind::Decimal,
            value,
        }
    }

    #[inline]
    pub fn kind(&self) -> IntegerKind {
        self.kind
    }

    #[inline]
    pub fn value(&self) -> i64 {
        self.value
    }
}

impl From<tombi_document_tree::Integer> for Integer {
    fn from(node: tombi_document_tree::Integer) -> Self {
        Self {
            kind: node.kind(),
            value: node.value(),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Integer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Integer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = i64::deserialize(deserializer)?;
        Ok(Self {
            kind: IntegerKind::Decimal,
            value,
        })
    }
}
