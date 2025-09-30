use crate::FromLsp;

pub trait IntoLsp<T> {
    fn into_lsp(self, line_index: &crate::LineIndex) -> T;
}

impl<T, U> IntoLsp<U> for T
where
    U: FromLsp<T>,
{
    fn into_lsp(self: T, line_index: &crate::LineIndex) -> U {
        U::from_lsp(self, line_index)
    }
}

#[cfg(test)]
mod tests {
    use crate::{features::lsp::WideEncoding, IntoLsp, LineIndex};

    #[test]
    fn test_ascii_into_lsp() {
        let line_index = LineIndex::new("hello\nworld", WideEncoding::Utf16);
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
        let line_index = LineIndex::new("🦅 Tombi", WideEncoding::Utf16);
        let range = crate::Range::from(((0, 0), (0, 1)));
        let expected_range = tower_lsp::lsp_types::Range {
            start: tower_lsp::lsp_types::Position::new(0, 0),
            end: tower_lsp::lsp_types::Position::new(0, 2),
        };

        let lsp_range: tower_lsp::lsp_types::Range = range.into_lsp(&line_index);
        pretty_assertions::assert_eq!(lsp_range, expected_range);
    }
}
