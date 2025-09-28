use unicode_segmentation::UnicodeSegmentation;

pub trait FromLsp {
    type Output;

    fn from_lsp(self, line_index: &crate::LineIndex) -> Self::Output;
}

impl FromLsp for tower_lsp::lsp_types::Position {
    type Output = crate::Position;

    fn from_lsp(self, line_index: &crate::LineIndex) -> Self::Output {
        let column = line_index
            .line_text(self.line)
            .map(|line_text| {
                let column_text =
                    take_column_text(line_text, self.character, line_index.wide_encoding);
                crate::WideEncoding::GraphemeCluster.measure(column_text)
            })
            .unwrap_or_default();

        crate::Position::new(self.line, column)
    }
}

fn take_column_text<'a>(
    line_text: &'a str,
    target_units: u32,
    encoding: crate::WideEncoding,
) -> &'a str {
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
    use crate::{LineIndex, Position, WideEncoding};

    #[test]
    fn converts_utf16_column_to_graphemes() {
        let text = "🦅 Tombi";
        let line_index = LineIndex::new(text, WideEncoding::Utf16);
        let lsp_position = tower_lsp::lsp_types::Position::new(0, 2);

        assert_eq!(lsp_position.from_lsp(&line_index), Position::new(0, 1));
    }

    #[test]
    fn clamps_when_lsp_column_exceeds_line() {
        let text = "hello";
        let line_index = LineIndex::new(text, WideEncoding::Utf8);
        let lsp_position = tower_lsp::lsp_types::Position::new(0, 10);

        assert_eq!(lsp_position.from_lsp(&line_index), Position::new(0, 5));
    }
}
