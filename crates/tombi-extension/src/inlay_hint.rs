#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlayHintKind {
    Type,
    Parameter,
}

impl InlayHintKind {
    pub const TYPE: Self = Self::Type;
    pub const PARAMETER: Self = Self::Parameter;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlayHint {
    pub position: tombi_text::Position,
    pub label: String,
    pub kind: Option<InlayHintKind>,
    pub tooltip: Option<String>,
    pub padding_left: Option<bool>,
    pub padding_right: Option<bool>,
}
