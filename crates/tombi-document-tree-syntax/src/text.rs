use std::{borrow::Borrow, fmt, hash::Hash, ops::Deref};

/// TOML text borrowed from the source or from the decoded-string pool.
///
/// Unescaped text points into the original source. Escaped text points into
/// the decoded pool for the document's TOML version.
#[derive(Clone, Copy)]
pub struct DocumentText<'t>(&'t str);

impl<'t> DocumentText<'t> {
    #[inline]
    pub(crate) fn try_new(
        syntax: &tombi_ast_syntax::SyntaxNode<'t>,
        resolver: &'t tombi_ast_syntax::DecodedTextResolver,
    ) -> Result<Self, tombi_toml_text::ParseError> {
        syntax.resolve_text(resolver).map(Self)
    }

    #[inline]
    pub(crate) fn new_raw(
        syntax: &tombi_ast_syntax::SyntaxNode<'t>,
        resolver: &tombi_ast_syntax::DecodedTextResolver,
    ) -> Self {
        Self(syntax.resolve_raw_text(resolver))
    }

    #[inline]
    pub fn as_str(&self) -> &'t str {
        self.0
    }

    #[inline]
    pub fn into_string(self) -> String {
        self.0.to_owned()
    }
}

impl Deref for DocumentText<'_> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for DocumentText<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for DocumentText<'_> {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Debug for DocumentText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl fmt::Display for DocumentText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl PartialEq for DocumentText<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for DocumentText<'_> {}

impl Hash for DocumentText<'_> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl PartialEq<str> for DocumentText<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for DocumentText<'_> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for DocumentText<'_> {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<DocumentText<'_>> for str {
    #[inline]
    fn eq(&self, other: &DocumentText<'_>) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<DocumentText<'_>> for &str {
    #[inline]
    fn eq(&self, other: &DocumentText<'_>) -> bool {
        *self == other.as_str()
    }
}

impl PartialEq<DocumentText<'_>> for String {
    #[inline]
    fn eq(&self, other: &DocumentText<'_>) -> bool {
        self == other.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::DocumentText;

    #[test]
    fn document_text_is_compact() {
        assert_eq!(std::mem::size_of::<DocumentText>(), 16);
    }
}
