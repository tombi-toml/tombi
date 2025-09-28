use unicode_segmentation::UnicodeSegmentation;

pub trait IntoLsp {
    type Output;

    fn into_lsp(self, line_index: &crate::LineIndex) -> Self::Output;
}

impl IntoLsp for crate::Position {
    type Output = tower_lsp::lsp_types::Position;

    fn into_lsp(self, line_index: &crate::LineIndex) -> Self::Output {
        let character = line_index
            .line_text(self.line)
            .map(|line_text| {
                line_text
                    .graphemes(true)
                    .take(self.column as usize)
                    .fold(0, |acc, char| acc + line_index.wide_encoding.measure(char))
            })
            .unwrap_or_default();

        tower_lsp::lsp_types::Position {
            line: self.line,
            character,
        }
    }
}

impl IntoLsp for crate::Range {
    type Output = tower_lsp::lsp_types::Range;

    fn into_lsp(self, line_index: &crate::LineIndex) -> Self::Output {
        tower_lsp::lsp_types::Range {
            start: self.start.into_lsp(line_index),
            end: self.end.into_lsp(line_index),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{IntoLsp, LineIndex, WideEncoding};

    #[test]
    fn test_ascii_into_lsp() {
        let line_index = LineIndex::new("hello\nworld", WideEncoding::Utf16);
        let range = crate::Range::from(((1, 2), (1, 3)));
        let expected_range = tower_lsp::lsp_types::Range {
            start: tower_lsp::lsp_types::Position::new(1, 2),
            end: tower_lsp::lsp_types::Position::new(1, 3),
        };

        let lsp_range = range.into_lsp(&line_index);
        pretty_assertions::assert_eq!(lsp_range, expected_range);
    }

    #[test]
    fn test_tombi_emoji_into_lsp() {
        let line_index = LineIndex::new("🦅 Tombi", WideEncoding::Utf16);
        let range = crate::Range::from(((0, 0), (0, 1)));
        let expected_range = tower_lsp::lsp_types::Range {
            start: tower_lsp::lsp_types::Position::new(0, 0),
            end: tower_lsp::lsp_types::Position::new(0, 2),
        };

        let lsp_range = range.into_lsp(&line_index);
        pretty_assertions::assert_eq!(lsp_range, expected_range);
    }
}
