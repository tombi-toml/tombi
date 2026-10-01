use crate::{LineIndex, Offset};

self_cell::self_cell!(
    struct OwnedLineIndexCell {
        owner: Box<str>,

        #[covariant]
        dependent: LineIndex,
    }
);

/// A [`LineIndex`] that owns the text it indexes.
///
/// A [`LineIndex`] borrows its text, so it cannot outlive the document it was built for.
/// This is for the text that is kept for a long time, such as a JSON Schema document.
pub struct OwnedLineIndex(OwnedLineIndexCell);

impl OwnedLineIndex {
    /// Computes the line index for `text`.
    pub fn new(text: impl Into<Box<str>>) -> Self {
        Self(OwnedLineIndexCell::new(text.into(), |text| {
            LineIndex::new(text)
        }))
    }

    /// Creates the line index from the start offsets of the lines already scanned in `text`.
    ///
    /// See [`LineIndex::from_line_starts`].
    pub fn from_line_starts(text: impl Into<Box<str>>, line_starts: Vec<Offset>) -> Self {
        Self(OwnedLineIndexCell::new(text.into(), |text| {
            LineIndex::from_line_starts(text, line_starts)
        }))
    }

    #[inline]
    pub fn as_line_index(&self) -> &LineIndex<'_> {
        self.0.borrow_dependent()
    }

    #[inline]
    pub fn text(&self) -> &str {
        self.0.borrow_owner()
    }
}

impl std::fmt::Debug for OwnedLineIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("OwnedLineIndex")
            .field(self.as_line_index())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{EncodingKind, Offset, Position, Range, Span};

    use super::OwnedLineIndex;

    #[test]
    fn converts_spans_of_the_owned_text() {
        let line_index = OwnedLineIndex::new("{\n  \"a\": 1\n}");

        assert_eq!(
            line_index.as_line_index().range(
                Span::new(Offset::new(4), Offset::new(7)),
                EncodingKind::Utf16
            ),
            Range::new(Position::new(1, 2), Position::new(1, 5))
        );
        assert_eq!(line_index.text(), "{\n  \"a\": 1\n}");
    }
}
