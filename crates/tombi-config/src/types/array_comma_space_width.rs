#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct ArrayCommaSpaceWidth(u8);

impl ArrayCommaSpaceWidth {
    #[inline]
    pub fn value(&self) -> u8 {
        self.0
    }
}

impl Default for ArrayCommaSpaceWidth {
    fn default() -> Self {
        Self(1)
    }
}

impl From<u8> for ArrayCommaSpaceWidth {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
