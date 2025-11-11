#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct TrailingCommentSpaceWidth(u8);

impl TrailingCommentSpaceWidth {
    #[inline]
    pub fn value(&self) -> u8 {
        self.0
    }
}

impl Default for TrailingCommentSpaceWidth {
    fn default() -> Self {
        Self(2)
    }
}

impl From<u8> for TrailingCommentSpaceWidth {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
