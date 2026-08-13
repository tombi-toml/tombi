use std::num::NonZeroU8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct LineWidth(NonZeroU8);

impl LineWidth {
    #[inline]
    pub fn value(&self) -> u8 {
        self.0.get()
    }
}

impl TryFrom<u8> for LineWidth {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        NonZeroU8::new(value)
            .map(Self)
            .ok_or("LineWidth must be a non-zero u8")
    }
}
