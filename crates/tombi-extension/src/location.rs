/// A location in a document.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Location {
    pub uri: tombi_uri::Uri,
    /// The range in the TOML document of `uri`, in the unit of the client's encoding.
    ///
    /// `None` opens the file at its start, for a file that is not parsed, such as a source file.
    pub range: Option<tombi_text::Range>,
}

/// A span in a document that is kept for a long time, such as a JSON Schema document,
/// with the line index to convert it.
#[derive(Debug, Clone)]
pub struct LocatedSpan {
    pub span: tombi_text::Span,
    pub line_index: std::sync::Arc<tombi_text::OwnedLineIndex>,
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
        self.line_index.as_line_index().range(self.span, encoding)
    }
}

/// Converts the spans of one document into ranges counted in the client's encoding.
///
/// An extension builds its output with it, because the output is where
/// a span turns into a line and a column.
#[derive(Debug, Clone, Copy)]
pub struct SpanConverter<'a, 'src> {
    line_index: &'a tombi_text::LineIndex<'src>,
    encoding: tombi_text::EncodingKind,
}

impl<'a, 'src> SpanConverter<'a, 'src> {
    pub fn new(
        line_index: &'a tombi_text::LineIndex<'src>,
        encoding: tombi_text::EncodingKind,
    ) -> Self {
        Self {
            line_index,
            encoding,
        }
    }

    #[inline]
    pub fn line_index(&self) -> &'a tombi_text::LineIndex<'src> {
        self.line_index
    }

    #[inline]
    pub fn encoding(&self) -> tombi_text::EncodingKind {
        self.encoding
    }

    #[inline]
    pub fn range(&self, span: tombi_text::Span) -> tombi_text::Range {
        self.line_index.range(span, self.encoding)
    }

    /// A location at `span` of the document converted by this converter.
    #[inline]
    pub fn location(&self, uri: tombi_uri::Uri, span: tombi_text::Span) -> Location {
        Location {
            uri,
            range: Some(self.range(span)),
        }
    }

    /// A text edit of the document converted by this converter.
    #[inline]
    pub fn text_edit(&self, edit: crate::TextEdit) -> crate::RangeTextEdit {
        crate::RangeTextEdit {
            range: self.range(edit.span),
            new_text: edit.new_text,
        }
    }
}

#[cfg(test)]
mod tests {
    use tombi_text::{EncodingKind, LineIndex, Offset, Position, Range, Span};

    use super::SpanConverter;

    #[test]
    fn converts_with_the_line_index_of_each_document() {
        let workspace_text = "e\u{301}x";
        let member_text = "👨‍👩‍👧‍👦x";
        let workspace_line_index = LineIndex::new(workspace_text);
        let member_line_index = LineIndex::new(member_text);

        // After the first grapheme cluster, counted in UTF-16 code units.
        let workspace = SpanConverter::new(&workspace_line_index, EncodingKind::Utf16);
        let member = SpanConverter::new(&member_line_index, EncodingKind::Utf16);

        assert_eq!(
            workspace.range(Span::empty(Offset::of(workspace_text.split_at(3).0))),
            Range::new(Position::new(0, 2), Position::new(0, 2))
        );
        assert_eq!(
            member.range(Span::empty(Offset::of("👨‍👩‍👧‍👦"))),
            Range::new(Position::new(0, 11), Position::new(0, 11))
        );
    }
}
