use std::{borrow::Borrow, fmt, hash::Hash, ops::Deref, sync::Arc};

/// TOML text backed by one shared immutable buffer.
///
/// Unescaped text points into the original source buffer. Escaped text points
/// into the decoded buffer for the document's TOML version. The span is local
/// to the selected buffer, so their combined length never shares one offset
/// space.
#[derive(Clone)]
pub struct DocumentText {
    buffer: Arc<Box<str>>,
    range: tombi_text::Span,
}

impl DocumentText {
    #[inline]
    pub(crate) fn try_new(
        syntax: &tombi_ast_syntax::SyntaxNode,
        resolver: &tombi_ast_syntax::DecodedTextResolver,
    ) -> Result<Self, tombi_toml_text::ParseError> {
        let (buffer, range) = syntax.resolve_text(resolver)?;
        Ok(Self { buffer, range })
    }

    #[inline]
    pub(crate) fn new_raw(
        syntax: &tombi_ast_syntax::SyntaxNode,
        resolver: &tombi_ast_syntax::DecodedTextResolver,
    ) -> Self {
        let (buffer, range) = syntax.resolve_raw_text(resolver);
        Self { buffer, range }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.buffer[self.range]
    }

    #[inline]
    pub fn into_string(self) -> String {
        self.as_str().to_owned()
    }
}

impl Deref for DocumentText {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for DocumentText {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for DocumentText {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Debug for DocumentText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl fmt::Display for DocumentText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl PartialEq for DocumentText {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for DocumentText {}

impl Hash for DocumentText {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl PartialEq<str> for DocumentText {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for DocumentText {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for DocumentText {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<DocumentText> for str {
    #[inline]
    fn eq(&self, other: &DocumentText) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<DocumentText> for &str {
    #[inline]
    fn eq(&self, other: &DocumentText) -> bool {
        *self == other.as_str()
    }
}

impl PartialEq<DocumentText> for String {
    #[inline]
    fn eq(&self, other: &DocumentText) -> bool {
        self == other.as_str()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::DocumentText;

    #[test]
    fn document_text_is_compact() {
        assert_eq!(std::mem::size_of::<DocumentText>(), 16);
    }

    #[test]
    fn document_text_span_is_local_to_its_buffer() {
        let range = tombi_text::Span::new(0.into(), 6.into());
        let source = DocumentText {
            buffer: Arc::new("source".into()),
            range,
        };
        let decoded = DocumentText {
            buffer: Arc::new("decoded".into()),
            range,
        };

        assert_eq!(source.as_str(), "source");
        assert_eq!(decoded.as_str(), "decode");
    }
}
