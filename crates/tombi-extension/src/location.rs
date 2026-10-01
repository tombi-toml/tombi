/// A location in a document.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Location {
    pub uri: tombi_uri::Uri,
    /// The span in the TOML document of `uri`.
    ///
    /// `None` opens the file at its start, for a file that is not parsed, such as a source file.
    pub span: Option<LocatedSpan>,
}

/// A span with the line index of its document, built while parsing it, to convert the span.
#[derive(Debug, Clone)]
pub struct LocatedSpan {
    pub span: tombi_text::Span,
    pub line_index: std::sync::Arc<tombi_text::LineIndex>,
}

/// Spans are compared without their line index, which is the same for the same document.
impl PartialEq for LocatedSpan {
    fn eq(&self, other: &Self) -> bool {
        self.span == other.span
    }
}

impl Eq for LocatedSpan {}

impl std::hash::Hash for LocatedSpan {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.span.hash(state);
    }
}

impl LocatedSpan {
    /// Converts the span into a range whose columns are counted in `encoding`.
    #[inline]
    pub fn range(&self, encoding: tombi_text::EncodingKind) -> tombi_text::Range {
        self.line_index.range(self.span, encoding)
    }
}
