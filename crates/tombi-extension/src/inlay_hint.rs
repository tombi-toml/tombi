use tombi_text::{FromLsp, IntoLsp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlayHint {
    pub position: tombi_text::Position,
    pub label: String,
    pub kind: Option<tower_lsp::lsp_types::InlayHintKind>,
    pub tooltip: Option<String>,
    pub padding_left: Option<bool>,
    pub padding_right: Option<bool>,
}

impl FromLsp<InlayHint> for tower_lsp::lsp_types::InlayHint {
    fn from_lsp(
        source: InlayHint,
        line_index: &tombi_text::LineIndex,
    ) -> tower_lsp::lsp_types::InlayHint {
        tower_lsp::lsp_types::InlayHint {
            position: source.position.into_lsp(line_index),
            label: source.label.into(),
            kind: source.kind,
            text_edits: None,
            tooltip: source.tooltip.map(Into::into),
            padding_left: source.padding_left,
            padding_right: source.padding_right,
            data: None,
        }
    }
}
