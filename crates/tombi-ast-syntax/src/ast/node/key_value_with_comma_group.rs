use tombi_ast_syntax::SyntaxNode;

use crate::AstNode;
use crate::support::iter::WithCommaIter;
use tombi_ast_syntax::SyntaxKind::KEY_VALUE_WITH_COMMA_GROUP;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyValueWithCommaGroup {
    pub(crate) syntax: SyntaxNode,
}

impl KeyValueWithCommaGroup {
    #[inline]
    pub fn key_values(&self) -> impl Iterator<Item = crate::KeyValue> {
        self.syntax()
            .child_nodes()
            .filter_map(crate::KeyValue::cast)
    }

    #[inline]
    pub fn into_key_values(self) -> impl Iterator<Item = crate::KeyValue> {
        self.syntax.child_nodes().filter_map(crate::KeyValue::cast)
    }

    #[inline]
    pub fn key_values_with_comma(
        &self,
    ) -> impl Iterator<Item = (crate::KeyValue, Option<crate::Comma>)> {
        WithCommaIter::new(self.syntax().child_nodes())
    }

    #[inline]
    pub fn into_key_values_with_comma(
        self,
    ) -> impl Iterator<Item = (crate::KeyValue, Option<crate::Comma>)> {
        WithCommaIter::new(self.syntax.child_nodes())
    }

    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

impl AstNode for KeyValueWithCommaGroup {
    #[inline]
    fn can_cast(kind: tombi_ast_syntax::SyntaxKind) -> bool {
        kind == KEY_VALUE_WITH_COMMA_GROUP
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
