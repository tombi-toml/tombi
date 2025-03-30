use crate::ToTomlString;

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

impl From<document_tree::Boolean> for Boolean {
    fn from(node: document_tree::Boolean) -> Self {
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

impl ToTomlString for Boolean {
    fn to_toml_string(&self, result: &mut std::string::String, _parent_keys: &[&crate::Key]) {
        result.push_str(if self.value { "true" } else { "false" });
    }
}
