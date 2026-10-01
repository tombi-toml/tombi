//! See [`LineIndex`].

use crate::{Column, EncodingKind, Line, Offset, Position, Range, Span};

/// Indexes the start offset of each line in a piece of text.
///
/// Text positions are kept as [`Span`]s, and converted to [`Range`]s with this index
/// only where a line and column are needed, such as diagnostics and LSP responses.
/// The unit of a column is chosen for each conversion by an [`EncodingKind`],
/// because the line breaks do not depend on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineIndex<'src> {
    text: &'src str,
    line_starts: Box<[Offset]>,
}

impl<'src> LineIndex<'src> {
    /// Computes the line index for `text`.
    pub fn new(text: &'src str) -> Self {
        let mut line_starts = vec![Offset::new(0)];
        line_starts.extend(
            memchr::memchr_iter(b'\n', text.as_bytes()).map(|index| offset_from_usize(index + 1)),
        );
        Self::from_line_starts(text, line_starts)
    }

    /// Creates the line index from the start offsets of the lines already scanned in `text`.
    ///
    /// `line_starts` must begin with `0`, followed by the offset after each `\n` in order.
    pub fn from_line_starts(text: &'src str, line_starts: Vec<Offset>) -> Self {
        debug_assert_eq!(line_starts.first(), Some(&Offset::new(0)));
        debug_assert_eq!(
            line_starts.len(),
            memchr::memchr_iter(b'\n', text.as_bytes()).count() + 1
        );
        Self {
            text,
            line_starts: line_starts.into_boxed_slice(),
        }
    }

    /// Returns the indexed text.
    #[inline]
    pub fn text(&self) -> &'src str {
        self.text
    }

    /// Returns the number of lines tracked by the index.
    #[inline]
    pub fn len(&self) -> usize {
        self.line_starts.len()
    }

    /// Returns true if no lines are tracked.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.line_starts.is_empty()
    }

    /// Returns the start offset of the line at `line`.
    #[inline]
    pub fn line_start(&self, line: Line) -> Option<Offset> {
        self.line_starts.get(line as usize).copied()
    }

    /// Returns the span of the line at `line`, excluding its line ending.
    pub fn line_span(&self, line: Line) -> Option<Span> {
        let line = line as usize;
        let start = *self.line_starts.get(line)?;
        Some(Span::new(start, self.line_end(line)))
    }

    /// Returns the text of the line at `line`, excluding its line ending.
    pub fn line_text(&self, line: Line) -> Option<&'src str> {
        self.line_span(line).map(|span| &self.text[span])
    }

    /// Returns an iterator over the spans of each line, excluding their line endings.
    pub fn iter(&self) -> impl Iterator<Item = Span> + '_ {
        (0..self.line_starts.len())
            .map(|line| Span::new(self.line_starts[line], self.line_end(line)))
    }

    /// Returns the line that contains `offset`.
    ///
    /// An offset past the end of the text belongs to the last line.
    #[inline]
    pub fn line(&self, offset: Offset) -> Line {
        (self.line_starts.partition_point(|start| *start <= offset) - 1) as Line
    }

    /// Converts `offset` into a position whose column is counted in `encoding`.
    ///
    /// An offset in a line ending is clamped to the end of the line.
    pub fn position(&self, offset: Offset, encoding: EncodingKind) -> Position {
        let line = self.line(offset);
        Position::new(line, self.column(line as usize, offset, encoding))
    }

    /// Converts `span` into a range whose columns are counted in `encoding`.
    pub fn range(&self, span: Span, encoding: EncodingKind) -> Range {
        let mut cursor = self.cursor(encoding);
        cursor.range(span)
    }

    /// Converts `position`, whose column is counted in `encoding`, into an offset.
    ///
    /// A column past the end of the line is clamped to the end of the line,
    /// and a line past the end of the text is clamped to the end of the text.
    pub fn offset(&self, position: Position, encoding: EncodingKind) -> Offset {
        let line = position.line as usize;
        let Some(&start) = self.line_starts.get(line) else {
            return offset_from_usize(self.text.len());
        };
        let line_text = &self.text[Span::new(start, self.line_end(line))];
        start + encoding.prefix_len(line_text, position.column) as u32
    }

    /// Converts `range`, whose columns are counted in `encoding`, into a span.
    pub fn span(&self, range: Range, encoding: EncodingKind) -> Span {
        Span::new(
            self.offset(range.start, encoding),
            self.offset(range.end, encoding),
        )
    }

    /// Returns a cursor that converts offsets in ascending order without searching the lines
    /// again for each offset.
    pub fn cursor(&self, encoding: EncodingKind) -> LineIndexCursor<'_, 'src> {
        LineIndexCursor {
            line_index: self,
            encoding,
            line: 0,
            offset: Offset::new(0),
            column: 0,
        }
    }

    /// The end of the line at `line`, excluding its line ending.
    fn line_end(&self, line: usize) -> Offset {
        let Some(&next_start) = self.line_starts.get(line + 1) else {
            return offset_from_usize(self.text.len());
        };
        let mut end = usize::from(next_start) - 1;
        if end > usize::from(self.line_starts[line]) && self.text.as_bytes()[end - 1] == b'\r' {
            end -= 1;
        }
        offset_from_usize(end)
    }

    fn column(&self, line: usize, offset: Offset, encoding: EncodingKind) -> Column {
        let start = self.line_starts[line];
        let offset = offset.min(self.line_end(line));
        encoding.measure(&self.text[Span::new(start, offset)])
    }
}

/// Converts offsets into positions, reusing the line and column of the previous offset.
///
/// Converting offsets in ascending order, as in a traversal of a document,
/// only measures the text between consecutive offsets.
/// An offset before the previous one is converted from the start of its line.
#[derive(Debug, Clone)]
pub struct LineIndexCursor<'a, 'src> {
    line_index: &'a LineIndex<'src>,
    encoding: EncodingKind,
    line: usize,
    offset: Offset,
    column: Column,
}

impl LineIndexCursor<'_, '_> {
    /// The unit of the columns this cursor counts.
    #[inline]
    pub fn encoding(&self) -> EncodingKind {
        self.encoding
    }

    /// Converts `offset` into a position.
    pub fn position(&mut self, offset: Offset) -> Position {
        let line_index = self.line_index;
        if offset < self.offset || !self.line_contains(self.line, offset) {
            self.line = if offset >= self.offset && self.line_contains(self.line + 1, offset) {
                self.line + 1
            } else {
                line_index.line(offset) as usize
            };
            self.offset = line_index.line_starts[self.line];
            self.column = 0;
        }

        let offset_in_line = offset.min(line_index.line_end(self.line));
        if offset_in_line > self.offset {
            self.column = if self.encoding == EncodingKind::GraphemeCluster {
                // Grapheme cluster widths don't add up across a split point
                // (a combining mark after it joins the cluster before it),
                // so the column is measured from the start of the line.
                line_index.column(self.line, offset_in_line, self.encoding)
            } else {
                self.column
                    + self
                        .encoding
                        .measure(&line_index.text[Span::new(self.offset, offset_in_line)])
            };
            self.offset = offset_in_line;
        }
        Position::new(self.line as Line, self.column)
    }

    /// Converts `span` into a range.
    pub fn range(&mut self, span: Span) -> Range {
        let start = self.position(span.start);
        Range::new(start, self.position(span.end))
    }

    /// Whether `offset` is in the line at `line`, including its line ending.
    fn line_contains(&self, line: usize, offset: Offset) -> bool {
        let starts = &self.line_index.line_starts;
        starts.get(line).is_some_and(|start| *start <= offset)
            && starts.get(line + 1).is_none_or(|next| offset < *next)
    }
}

#[inline]
fn offset_from_usize(value: usize) -> Offset {
    debug_assert!(value <= u32::MAX as usize, "text is too long to index");
    Offset::new(value as u32)
}

#[cfg(test)]
mod tests {
    use crate::{EncodingKind, Offset, Position, Range, Span};

    use super::LineIndex;

    fn lines(text: &str) -> Vec<&str> {
        let index = LineIndex::new(text);
        index.iter().map(|span| &text[span]).collect()
    }

    #[test]
    fn indexes_unix_newlines() {
        assert_eq!(lines("foo\nbar\nbaz"), ["foo", "bar", "baz"]);
    }

    #[test]
    fn indexes_trailing_newline() {
        assert_eq!(lines("foo\n"), ["foo", ""]);
    }

    #[test]
    fn indexes_windows_newlines() {
        assert_eq!(lines("foo\r\nbar\r\n"), ["foo", "bar", ""]);
    }

    #[test]
    fn converts_offsets_in_each_encoding() {
        let text = "a = 1\n🦅 = \"👨‍👩‍👧\"\n";
        let index = LineIndex::new(text);
        let offset = Offset::of(&text[..text.find("\"").unwrap()]);

        assert_eq!(
            index.position(offset, EncodingKind::Utf8),
            Position::new(1, 7)
        );
        assert_eq!(
            index.position(offset, EncodingKind::Utf16),
            Position::new(1, 5)
        );
        assert_eq!(
            index.position(offset, EncodingKind::Utf32),
            Position::new(1, 4)
        );
        assert_eq!(
            index.position(offset, EncodingKind::GraphemeCluster),
            Position::new(1, 4)
        );

        for encoding in [
            EncodingKind::Utf8,
            EncodingKind::Utf16,
            EncodingKind::Utf32,
            EncodingKind::GraphemeCluster,
        ] {
            assert_eq!(
                index.offset(index.position(offset, encoding), encoding),
                offset
            );
        }
    }

    #[test]
    fn clamps_offsets_in_line_endings() {
        let index = LineIndex::new("ab\r\ncd");
        assert_eq!(
            index.position(Offset::new(3), EncodingKind::Utf16),
            Position::new(0, 2)
        );
    }

    #[test]
    fn clamps_positions_past_the_line() {
        let index = LineIndex::new("ab\ncd");
        assert_eq!(
            index.offset(Position::new(0, 10), EncodingKind::Utf16),
            Offset::new(2)
        );
        assert_eq!(
            index.offset(Position::new(5, 0), EncodingKind::Utf16),
            Offset::new(5)
        );
    }

    #[test]
    fn cursor_matches_random_access() {
        // "e\u{301}" is one grapheme cluster split by a char boundary.
        let text = "a = \"é\"\n\n🦅 = [1, \"👨‍👩‍👧\", \"e\u{301}\"]\r\nb = 2\n";
        let index = LineIndex::new(text);
        let offsets = (0..=text.len())
            .filter(|offset| text.is_char_boundary(*offset))
            .map(|offset| Offset::new(offset as u32))
            .collect::<Vec<_>>();

        for encoding in [
            EncodingKind::Utf8,
            EncodingKind::Utf16,
            EncodingKind::Utf32,
            EncodingKind::GraphemeCluster,
        ] {
            let mut cursor = index.cursor(encoding);
            for offset in &offsets {
                assert_eq!(cursor.position(*offset), index.position(*offset, encoding));
            }
            // Offsets that go backwards are converted from the start of their line.
            for offset in offsets.iter().rev() {
                assert_eq!(cursor.position(*offset), index.position(*offset, encoding));
            }
        }
    }

    #[test]
    fn converts_spans_and_ranges() {
        let text = "key = \"値\"\n";
        let index = LineIndex::new(text);
        let span = Span::new(Offset::new(6), Offset::new(11));
        let range = Range::new(Position::new(0, 6), Position::new(0, 9));

        assert_eq!(index.range(span, EncodingKind::Utf16), range);
        assert_eq!(index.span(range, EncodingKind::Utf16), span);
    }
}
