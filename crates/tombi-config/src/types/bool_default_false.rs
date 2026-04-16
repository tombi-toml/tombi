#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("default" = false)))]
pub struct BoolDefaultFalse(pub bool);

impl BoolDefaultFalse {
    #[inline]
    pub fn value(&self) -> bool {
        self.0
    }
}

impl Default for BoolDefaultFalse {
    fn default() -> Self {
        Self(false)
    }
}

impl From<bool> for BoolDefaultFalse {
    fn from(value: bool) -> Self {
        Self(value)
    }
}
