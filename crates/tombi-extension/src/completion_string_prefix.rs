use tombi_text::{Offset, Span};
use tombi_toml_version::TomlVersion;

/// A decoded TOML string prefix and the source spelling used for completion filtering.
pub struct CompletionStringPrefix {
    value: String,
    source: String,
    literal: bool,
}

impl CompletionStringPrefix {
    pub fn parse(text: &str, span: Span, offset: Offset, version: TomlVersion) -> Option<Self> {
        let source = &text[span];
        let delimiter = if source.starts_with("\"\"\"") {
            "\"\"\""
        } else if source.starts_with("'''") {
            "'''"
        } else if source.starts_with('\'') {
            "'"
        } else if source.starts_with('"') {
            "\""
        } else {
            return None;
        };
        let raw = &text[Span::new(span.start, offset.min(span.end))];
        let content = raw.strip_prefix(delimiter)?;
        let content = if offset == span.end && raw.len() >= 2 * delimiter.len() {
            content.strip_suffix(delimiter).unwrap_or(content)
        } else {
            content
        };
        let multiline = delimiter.len() == 3;
        let content = if multiline {
            content
                .strip_prefix("\r\n")
                .or_else(|| content.strip_prefix('\n'))
                .unwrap_or(content)
        } else {
            content
        };
        let literal = delimiter.starts_with('\'');
        let value = if literal {
            tombi_toml_text::parse_literal_string(content, multiline)
        } else {
            tombi_toml_text::parse_basic_string(content, version, multiline)
        }
        .map(std::borrow::Cow::into_owned)
        .unwrap_or_else(|_| content.to_owned());
        Some(Self {
            value,
            source: raw.to_owned(),
            literal,
        })
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn filter_text_for_suffix(&self, suffix: &str) -> String {
        let suffix = if self.literal {
            suffix.to_owned()
        } else {
            let quoted = serde_json::Value::String(suffix.to_owned()).to_string();
            quoted[1..quoted.len() - 1].to_owned()
        };
        format!("{}{suffix}", self.source)
    }
}
