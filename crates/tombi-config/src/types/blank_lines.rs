use std::num::NonZeroU8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct BlankLines(u8);

impl BlankLines {
    #[inline]
    pub fn value(&self) -> u8 {
        self.0
    }
}

impl Default for BlankLines {
    fn default() -> Self {
        Self(1)
    }
}

impl TryFrom<u8> for BlankLines {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct BlankLinesLimit(NonZeroU8);

impl BlankLinesLimit {
    #[inline]
    pub fn value(&self) -> u8 {
        self.0.get()
    }
}

impl Default for BlankLinesLimit {
    fn default() -> Self {
        Self(NonZeroU8::new(1).unwrap())
    }
}

impl TryFrom<u8> for BlankLinesLimit {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        NonZeroU8::new(value)
            .map(Self)
            .ok_or("BlankLinesLimit must be a non-zero u8")
    }
}
