use tombi_ast_syntax::SyntaxNode;

use crate::AstNode;
use crate::support::iter::WithCommaIter;
use tombi_ast_syntax::SyntaxKind::VALUE_WITH_COMMA_GROUP;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValueWithCommaGroup<'t> {
    pub(crate) syntax: SyntaxNode<'t>,
}

impl<'t> ValueWithCommaGroup<'t> {
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = crate::Value<'t>> {
        self.syntax().child_nodes().filter_map(crate::Value::cast)
    }

    #[inline]
    pub fn into_values(self) -> impl Iterator<Item = crate::Value<'t>> {
        self.syntax.child_nodes().filter_map(crate::Value::cast)
    }

    #[inline]
    pub fn values_with_comma(
        &self,
    ) -> impl Iterator<Item = (crate::Value<'t>, Option<crate::Comma<'t>>)> {
        WithCommaIter::new(self.syntax().child_nodes())
    }

    #[inline]
    pub fn into_values_with_comma(
        self,
    ) -> impl Iterator<Item = (crate::Value<'t>, Option<crate::Comma<'t>>)> {
        WithCommaIter::new(self.syntax.child_nodes())
    }

    #[inline]
    pub fn value_or_key_values_with_comma(
        &self,
    ) -> impl Iterator<Item = (crate::ValueOrKeyValue<'t>, Option<crate::Comma<'t>>)> {
        WithCommaIter::new(self.syntax().child_nodes())
    }

    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.syntax.span()
    }
}

impl<'t> AstNode<'t> for ValueWithCommaGroup<'t> {
    #[inline]
    fn can_cast(kind: tombi_ast_syntax::SyntaxKind) -> bool {
        kind == VALUE_WITH_COMMA_GROUP
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
