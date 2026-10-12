use crate::{EncodingKind, FromLsp, LineIndex};

pub trait IntoLsp<Output> {
    fn into_lsp(self, line_index: &LineIndex, encoding: EncodingKind) -> Output;
}

impl<Input, Output> IntoLsp<Output> for Input
where
    Output: FromLsp<Input>,
{
    fn into_lsp(self: Input, line_index: &LineIndex, encoding: EncodingKind) -> Output {
        Output::from_lsp(self, line_index, encoding)
    }
}

#[cfg(test)]
mod tests {
    use crate::{EncodingKind, IntoLsp, LineIndex, Offset, Span};

    #[test]
    fn test_ascii_into_lsp() {
        let line_index = LineIndex::new("hello\nworld");
        let span = Span::new(Offset::new(8), Offset::new(9));
        let expected_range = tower_lsp_server::gen_lsp_types::Range {
            start: tower_lsp_server::gen_lsp_types::Position::new(1, 2),
            end: tower_lsp_server::gen_lsp_types::Position::new(1, 3),
        };

        let lsp_range: tower_lsp_server::gen_lsp_types::Range =
            span.into_lsp(&line_index, EncodingKind::Utf16);
        pretty_assertions::assert_eq!(lsp_range, expected_range);
    }

    #[test]
    fn test_tombi_emoji_into_lsp() {
        let line_index = LineIndex::new("🦅 Tombi");
        let span = Span::new(Offset::new(0), Offset::new(4));
        let expected_range = tower_lsp_server::gen_lsp_types::Range {
            start: tower_lsp_server::gen_lsp_types::Position::new(0, 0),
            end: tower_lsp_server::gen_lsp_types::Position::new(0, 2),
        };

        let lsp_range: tower_lsp_server::gen_lsp_types::Range =
            span.into_lsp(&line_index, EncodingKind::Utf16);
        pretty_assertions::assert_eq!(lsp_range, expected_range);
    }
}
