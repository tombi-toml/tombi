#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum LineEnding {
    #[default]
    Lf,
    Crlf,
}

impl From<LineEnding> for &'static str {
    fn from(val: LineEnding) -> Self {
        match val {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
        }
    }
}
