#[derive(Debug, Clone)]
pub struct AnythingSchema {
    pub title: Option<String>,
    pub description: Option<String>,
    pub span: tombi_text::Span,
}
