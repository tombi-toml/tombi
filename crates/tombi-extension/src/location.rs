#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Location {
    pub uri: tombi_uri::Uri,
    pub range: tombi_text::Range,
}
