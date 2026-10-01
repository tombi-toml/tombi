use crate::{EncodingKind, LineIndex};

/// Converts between tombi's text positions and LSP's, whose columns are counted in `encoding`.
pub trait FromLsp<Input> {
    fn from_lsp(source: Input, line_index: &LineIndex, encoding: EncodingKind) -> Self;
}

impl FromLsp<tower_lsp::lsp_types::Position> for crate::Offset {
    fn from_lsp(
        source: tower_lsp::lsp_types::Position,
        line_index: &LineIndex,
        encoding: EncodingKind,
    ) -> Self {
        line_index.offset(
            crate::Position::new(source.line, source.character),
            encoding,
        )
    }
}

impl FromLsp<tower_lsp::lsp_types::Range> for crate::Span {
    fn from_lsp(
        source: tower_lsp::lsp_types::Range,
        line_index: &LineIndex,
        encoding: EncodingKind,
    ) -> Self {
        crate::Span::new(
            crate::Offset::from_lsp(source.start, line_index, encoding),
            crate::Offset::from_lsp(source.end, line_index, encoding),
        )
    }
}

impl FromLsp<crate::Offset> for tower_lsp::lsp_types::Position {
    fn from_lsp(source: crate::Offset, line_index: &LineIndex, encoding: EncodingKind) -> Self {
        line_index.cursor(encoding).lsp_position(source)
    }
}

impl FromLsp<crate::Span> for tower_lsp::lsp_types::Range {
    fn from_lsp(source: crate::Span, line_index: &LineIndex, encoding: EncodingKind) -> Self {
        line_index.cursor(encoding).lsp_range(source)
    }
}

impl crate::LineIndexCursor<'_, '_> {
    /// Converts `offset` into an LSP position, whose column is counted in the encoding of
    /// the cursor.
    ///
    /// A [`crate::Position`] does not know the unit of its column, so it is not converted into
    /// an LSP position directly.
    pub fn lsp_position(&mut self, offset: crate::Offset) -> tower_lsp::lsp_types::Position {
        debug_assert_ne!(
            self.encoding(),
            EncodingKind::GraphemeCluster,
            "LSP does not count columns in grapheme clusters"
        );
        let position = self.position(offset);
        tower_lsp::lsp_types::Position::new(position.line, position.column)
    }

    /// Converts `span` into an LSP range, whose columns are counted in the encoding of the cursor.
    pub fn lsp_range(&mut self, span: crate::Span) -> tower_lsp::lsp_types::Range {
        let start = self.lsp_position(span.start);
        tower_lsp::lsp_types::Range::new(start, self.lsp_position(span.end))
    }
}

#[cfg(test)]
mod tests {
    use super::FromLsp;
    use crate::{EncodingKind, LineIndex, Offset};

    #[test]
    fn converts_utf16_column_to_offset() {
        let line_index = LineIndex::new("🦅 Tombi");
        let lsp_position = tower_lsp::lsp_types::Position::new(0, 2);

        assert_eq!(
            Offset::from_lsp(lsp_position, &line_index, EncodingKind::Utf16),
            Offset::new(4)
        );
    }

    #[test]
    fn clamps_when_lsp_column_exceeds_line() {
        let line_index = LineIndex::new("hello");
        let lsp_position = tower_lsp::lsp_types::Position::new(0, 10);

        assert_eq!(
            Offset::from_lsp(lsp_position, &line_index, EncodingKind::Utf8),
            Offset::new(5)
        );
    }
}
