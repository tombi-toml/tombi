//! See [`LineIndex`].

use crate::{features::lsp::EncodingKind, Offset, Span};

/// Indexes the start and end offsets of each line in a piece of text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineIndex<'a> {
    text: &'a str,
    lines: Vec<Span>,
    pub encoding_kind: EncodingKind,
}

impl<'a> LineIndex<'a> {
    /// Computes the line index for `text`.
    pub fn new(text: &'a str, encoding_kind: EncodingKind) -> Self {
        let mut lines = Vec::new();
        let mut start: usize = 0;
        let bytes = text.as_bytes();

        for (idx, ch) in text.char_indices() {
            if ch == '\n' {
                let line_end = if idx > start && bytes.get(idx - 1) == Some(&b'\r') {
                    idx - 1
                } else {
                    idx
                };

                lines.push(Span::new(
                    offset_from_usize(start),
                    offset_from_usize(line_end),
                ));

                start = idx + ch.len_utf8();
            }
        }

        let final_end = text.len();
        lines.push(Span::new(
            offset_from_usize(start),
            offset_from_usize(final_end),
        ));

        LineIndex {
            text,
            lines,
            encoding_kind,
        }
    }

    /// Returns the number of lines tracked by the index.
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Returns true if no lines are tracked.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Returns the span for the line at `line_idx`, if it exists.
    pub fn line_text(&self, line_idx: crate::Line) -> Option<&'a str> {
        self.lines
            .get(line_idx as usize)
            .map(|span| &self.text[usize::from(span.start)..usize::from(span.end)])
    }

    /// Returns an iterator over the spans of each line.
    pub fn iter(&self) -> impl Iterator<Item = Span> + '_ {
        self.lines.iter().copied()
    }
}

#[inline]
fn offset_from_usize(value: usize) -> Offset {
    debug_assert!(value <= u32::MAX as usize, "text is too long to index");
    Offset::new(value as u32)
}

#[cfg(test)]
mod tests {
    use crate::features::lsp::EncodingKind;

    use super::LineIndex;

    #[test]
    fn indexes_unix_newlines() {
        let text = "foo\nbar\nbaz";
        let index = LineIndex::new(text, EncodingKind::Utf8);

        let lines: Vec<&str> = index
            .iter()
            .map(|span| &text[usize::from(span.start)..usize::from(span.end)])
            .collect();

        assert_eq!(lines, ["foo", "bar", "baz"]);
    }

    #[test]
    fn indexes_trailing_newline() {
        let text = "foo\n";
        let index = LineIndex::new(text, EncodingKind::Utf8);

        let lines: Vec<&str> = index
            .iter()
            .map(|span| &text[usize::from(span.start)..usize::from(span.end)])
            .collect();

        assert_eq!(lines, ["foo", ""]);
    }

    #[test]
    fn indexes_windows_newlines() {
        let text = "foo\r\nbar";
        let index = LineIndex::new(text, EncodingKind::Utf8);

        let lines: Vec<&str> = index
            .iter()
            .map(|span| &text[usize::from(span.start)..usize::from(span.end)])
            .collect();

        assert_eq!(lines, ["foo", "bar"]);
    }
}
