#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct KeyValueEqualsSignSpaceWidth(u8);

impl KeyValueEqualsSignSpaceWidth {
    #[inline]
    pub fn value(&self) -> u8 {
        self.0
    }
}

impl Default for KeyValueEqualsSignSpaceWidth {
    fn default() -> Self {
        Self(1)
    }
}

impl From<u8> for KeyValueEqualsSignSpaceWidth {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
