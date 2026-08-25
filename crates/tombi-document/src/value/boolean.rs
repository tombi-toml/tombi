#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Boolean {
    value: bool,
}

impl Boolean {
    #[inline]
    pub fn new(value: bool) -> Self {
        Self { value }
    }

    #[inline]
    pub fn value(&self) -> bool {
        self.value
    }
}

impl From<tombi_document_tree_syntax::Boolean> for Boolean {
    fn from(node: tombi_document_tree_syntax::Boolean) -> Self {
        Self {
            value: node.value(),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Boolean {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Boolean {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        bool::deserialize(deserializer).map(|value| Self { value })
    }
}
