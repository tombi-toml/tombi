use tombi_ast_syntax::SyntaxNode;

use crate::AstNode;
use crate::support::iter::WithCommaIter;
use tombi_ast_syntax::SyntaxKind::KEY_VALUE_WITH_COMMA_GROUP;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyValueWithCommaGroup<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}

impl<'t> KeyValueWithCommaGroup<'t> {
    #[inline]
    pub fn key_values(&self) -> impl Iterator<Item = crate::KeyValue<'t>> {
        self.syntax()
            .child_nodes()
            .filter_map(crate::KeyValue::cast)
    }

    #[inline]
    pub fn into_key_values(self) -> impl Iterator<Item = crate::KeyValue<'t>> {
        self.syntax.child_nodes().filter_map(crate::KeyValue::cast)
    }

    #[inline]
    pub fn key_values_with_comma(
        &self,
    ) -> impl Iterator<Item = (crate::KeyValue<'t>, Option<crate::Comma<'t>>)> {
        WithCommaIter::new(self.syntax().child_nodes())
    }

    #[inline]
    pub fn into_key_values_with_comma(
        self,
    ) -> impl Iterator<Item = (crate::KeyValue<'t>, Option<crate::Comma<'t>>)> {
        WithCommaIter::new(self.syntax.child_nodes())
    }

    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

impl<'t> AstNode<'t> for KeyValueWithCommaGroup<'t> {
    #[inline]
    fn can_cast(kind: tombi_ast_syntax::SyntaxKind) -> bool {
        kind == KEY_VALUE_WITH_COMMA_GROUP
    }

    #[inline]
    fn cast(syntax: SyntaxNode<'t>) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    #[inline]
    fn syntax(&self) -> &SyntaxNode<'t> {
        &self.syntax
    }
}
