#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct ArrayBracketSpaceWidth(u8);

impl ArrayBracketSpaceWidth {
    #[inline]
    pub fn value(&self) -> u8 {
        self.0
    }
}

impl Default for ArrayBracketSpaceWidth {
    fn default() -> Self {
        Self(0)
    }
}

impl From<u8> for ArrayBracketSpaceWidth {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
