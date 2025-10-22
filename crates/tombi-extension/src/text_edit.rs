/// General-purpose type for representing LSP text edits that require range conversion.
/// Uses `tombi_text::Range` internally and converts to LSP type via the `FromLsp` trait.
use tombi_text::IntoLsp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    pub range: tombi_text::Range,
    pub new_text: String,
}

impl tombi_text::FromLsp<TextEdit> for tower_lsp::lsp_types::TextEdit {
    fn from_lsp(
        source: TextEdit,
        line_index: &tombi_text::LineIndex,
    ) -> tower_lsp::lsp_types::TextEdit {
        tower_lsp::lsp_types::TextEdit {
            range: source.range.into_lsp(line_index),
            new_text: source.new_text,
        }
    }
}
