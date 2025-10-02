use crate::FromLsp;

pub trait IntoLsp<Output> {
    fn into_lsp(self, line_index: &crate::LineIndex) -> Output;
}

impl<Input, Output> IntoLsp<Output> for Input
where
    Output: FromLsp<Input>,
{
    fn into_lsp(self: Input, line_index: &crate::LineIndex) -> Output {
        Output::from_lsp(self, line_index)
    }
}

#[cfg(test)]
mod tests {
    use crate::{features::lsp::EncodingKind, IntoLsp, LineIndex};

    #[test]
    fn test_ascii_into_lsp() {
        let line_index = LineIndex::new("hello\nworld", EncodingKind::Utf16);
        let range = crate::Range::from(((1, 2), (1, 3)));
        let expected_range = tower_lsp::lsp_types::Range {
            start: tower_lsp::lsp_types::Position::new(1, 2),
            end: tower_lsp::lsp_types::Position::new(1, 3),
        };

        let lsp_range: tower_lsp::lsp_types::Range = range.into_lsp(&line_index);
        pretty_assertions::assert_eq!(lsp_range, expected_range);
    }

    #[test]
    fn test_tombi_emoji_into_lsp() {
        let line_index = LineIndex::new("🦅 Tombi", EncodingKind::Utf16);
        let range = crate::Range::from(((0, 0), (0, 1)));
        let expected_range = tower_lsp::lsp_types::Range {
            start: tower_lsp::lsp_types::Position::new(0, 0),
            end: tower_lsp::lsp_types::Position::new(0, 2),
        };

        let lsp_range: tower_lsp::lsp_types::Range = range.into_lsp(&line_index);
        pretty_assertions::assert_eq!(lsp_range, expected_range);
    }
}
