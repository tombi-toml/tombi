#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum LineEnding {
    #[default]
    Auto,
    Lf,
    Crlf,
}

impl LineEnding {
    /// Resolve the line ending to a concrete string based on the source text.
    ///
    /// For `Auto`, detects the line ending from the source: if `\r\n` is found, uses CRLF;
    /// otherwise defaults to LF.
    pub fn resolve(self, source: &str) -> &'static str {
        match self {
            LineEnding::Auto => {
                if source.contains("\r\n") {
                    "\r\n"
                } else {
                    "\n"
                }
            }
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
        }
    }
}

impl From<LineEnding> for &'static str {
    fn from(val: LineEnding) -> Self {
        match val {
            LineEnding::Auto | LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
        }
    }
}
