#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    pub range: tombi_text::Range,
    pub new_text: String,
}
