#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct InlineTableBraceSpaceWidth(u8);

impl InlineTableBraceSpaceWidth {
    #[inline]
    pub fn value(&self) -> u8 {
        self.0
    }
}

impl Default for InlineTableBraceSpaceWidth {
    fn default() -> Self {
        Self(1)
    }
}

impl From<u8> for InlineTableBraceSpaceWidth {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
