#[derive(Debug, Clone)]
pub struct AnythingSchema {
    pub title: Option<String>,
    pub description: Option<String>,
    pub range: tombi_text::Range,
}
