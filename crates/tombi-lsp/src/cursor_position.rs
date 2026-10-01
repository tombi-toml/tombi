/// The position of the cursor of a request in a document.
///
/// Besides its offset, it answers questions about its line, which a span cannot.
/// The line of the cursor is looked up once, when the position is created.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CursorPosition<'a> {
    offset: tombi_text::Offset,
    line: tombi_text::Line,
    /// The start of the line of the cursor.
    line_start: tombi_text::Offset,
    /// The start of the next line, or `None` on the last line.
    next_line_start: Option<tombi_text::Offset>,
    line_index: &'a tombi_text::LineIndex<'a>,
}

impl<'a> CursorPosition<'a> {
    pub(crate) fn new(
        offset: tombi_text::Offset,
        line_index: &'a tombi_text::LineIndex<'a>,
    ) -> Self {
        let line = line_index.line(offset);
        Self {
            offset,
            line,
            line_start: line_index.line_start(line).unwrap_or_default(),
            next_line_start: line_index.line_start(line + 1),
            line_index,
        }
    }

    #[inline]
    pub(crate) fn offset(self) -> tombi_text::Offset {
        self.offset
    }

    /// Whether `offset` is on the line of the cursor, including its line ending.
    #[inline]
    pub(crate) fn is_on_line(self, offset: tombi_text::Offset) -> bool {
        self.line_start <= offset && self.next_line_start.is_none_or(|next| offset < next)
    }

    /// The number of line breaks from `offset` to the cursor, and the column of the cursor
    /// in grapheme clusters, to keep the layout before the cursor.
    pub(crate) fn line_breaks_and_column_from(self, offset: tombi_text::Offset) -> (u32, u32) {
        let line_breaks = self.line.saturating_sub(self.line_index.line(offset));
        let column = self
            .line_index
            .position(self.offset, tombi_text::EncodingKind::GraphemeCluster)
            .column;
        (line_breaks, column)
    }
}
