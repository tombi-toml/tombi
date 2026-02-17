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
    /// For `Auto`, the newline style is detected automatically on a file-by-file basis.
    /// Files with mixed line endings will be converted to the first detected line ending.
    /// Defaults to `\n` for files that contain no line endings.
    pub fn resolve(self, source: &str) -> &'static str {
        match self {
            LineEnding::Auto => {
                // Check the first newline to determine the line ending style.
                match source.find('\n') {
                    Some(pos) if pos > 0 && source.as_bytes()[pos - 1] == b'\r' => "\r\n",
                    _ => "\n",
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
