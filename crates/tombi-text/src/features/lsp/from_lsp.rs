use unicode_segmentation::UnicodeSegmentation;

use crate::{features::lsp::EncodingKind, IntoLsp};

pub trait FromLsp<Input> {
    fn from_lsp(source: Input, line_index: &crate::LineIndex) -> Self;
}

impl FromLsp<tower_lsp::lsp_types::Position> for crate::Position {
    fn from_lsp(
        source: tower_lsp::lsp_types::Position,
        line_index: &crate::LineIndex,
    ) -> crate::Position {
        let column = line_index
            .line_text(source.line)
            .map(|line_text| {
                let column_text =
                    take_column_text(line_text, source.character, line_index.encoding_kind);
                EncodingKind::GraphemeCluster.measure(column_text)
            })
            .unwrap_or_default();

        crate::Position::new(source.line, column)
    }
}

impl FromLsp<crate::Position> for tower_lsp::lsp_types::Position {
    fn from_lsp(
        source: crate::Position,
        line_index: &crate::LineIndex,
    ) -> tower_lsp::lsp_types::Position {
        let character = line_index
            .line_text(source.line)
            .map(|line_text| {
                line_text
                    .graphemes(true)
                    .take(source.column as usize)
                    .fold(0, |acc, char| acc + line_index.encoding_kind.measure(char))
            })
            .unwrap_or_default();

        tower_lsp::lsp_types::Position {
            line: source.line,
            character,
        }
    }
}

impl FromLsp<tower_lsp::lsp_types::Range> for crate::Range {
    fn from_lsp(
        source: tower_lsp::lsp_types::Range,
        line_index: &crate::LineIndex,
    ) -> crate::Range {
        crate::Range::new(
            crate::Position::from_lsp(source.start, line_index),
            crate::Position::from_lsp(source.end, line_index),
        )
    }
}

impl FromLsp<crate::Range> for tower_lsp::lsp_types::Range {
    fn from_lsp(
        source: crate::Range,
        line_index: &crate::LineIndex,
    ) -> tower_lsp::lsp_types::Range {
        tower_lsp::lsp_types::Range {
            start: source.start.into_lsp(line_index),
            end: source.end.into_lsp(line_index),
        }
    }
}

fn take_column_text(line_text: &str, target_units: u32, encoding: EncodingKind) -> &str {
    if target_units == 0 {
        return "";
    }

    let mut consumed_units: u32 = 0;

    for (offset, grapheme) in UnicodeSegmentation::grapheme_indices(line_text, true) {
        let width = encoding.measure(grapheme);
        let next_units = consumed_units.saturating_add(width);

        if next_units > target_units {
            return &line_text[..offset];
        }

        consumed_units = next_units;

        if consumed_units == target_units {
            let end = offset + grapheme.len();
            return &line_text[..end];
        }
    }

    line_text
}

#[cfg(test)]
mod tests {
    use super::FromLsp;
    use crate::{features::lsp::EncodingKind, LineIndex, Position};

    #[test]
    fn converts_utf16_column_to_graphemes() {
        let text = "🦅 Tombi";
        let line_index = LineIndex::new(text, EncodingKind::Utf16);
        let lsp_position = tower_lsp::lsp_types::Position::new(0, 2);

        assert_eq!(
            Position::from_lsp(lsp_position, &line_index),
            Position::new(0, 1)
        );
    }

    #[test]
    fn clamps_when_lsp_column_exceeds_line() {
        let text = "hello";
        let line_index = LineIndex::new(text, EncodingKind::Utf8);
        let lsp_position = tower_lsp::lsp_types::Position::new(0, 10);

        assert_eq!(
            Position::from_lsp(lsp_position, &line_index),
            Position::new(0, 5)
        );
    }
}
