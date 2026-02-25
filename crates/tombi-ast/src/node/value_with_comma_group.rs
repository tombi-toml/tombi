use tombi_syntax::SyntaxNode;

use crate::AstNode;
use crate::support::iter::WithCommaIter;
use tombi_syntax::SyntaxKind::VALUE_WITH_COMMA_GROUP;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValueWithCommaGroup {
    pub(crate) syntax: SyntaxNode,
}

impl ValueWithCommaGroup {
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = crate::Value> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|el| el.into_node().and_then(crate::Value::cast))
    }

    #[inline]
    pub fn into_values(self) -> impl Iterator<Item = crate::Value> {
        self.syntax
            .children_with_tokens()
            .filter_map(|el| el.into_node().and_then(crate::Value::cast))
    }

    #[inline]
    pub fn values_with_comma(&self) -> impl Iterator<Item = (crate::Value, Option<crate::Comma>)> {
        WithCommaIter::new(self.syntax().children())
    }

    #[inline]
    pub fn into_values_with_comma(
        self,
    ) -> impl Iterator<Item = (crate::Value, Option<crate::Comma>)> {
        WithCommaIter::new(self.syntax.children())
    }

    #[inline]
    pub fn value_or_key_values_with_comma(
        &self,
    ) -> impl Iterator<Item = (crate::ValueOrKeyValue, Option<crate::Comma>)> {
        WithCommaIter::new(self.syntax().children())
    }

    #[inline]
    pub fn into_value_or_key_values_with_comma(
        self,
    ) -> impl Iterator<Item = (crate::ValueOrKeyValue, Option<crate::Comma>)> {
        WithCommaIter::new(self.syntax.children())
    }

    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

impl AstNode for ValueWithCommaGroup {
    #[inline]
    fn can_cast(kind: tombi_syntax::SyntaxKind) -> bool {
        kind == VALUE_WITH_COMMA_GROUP
    }

    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
