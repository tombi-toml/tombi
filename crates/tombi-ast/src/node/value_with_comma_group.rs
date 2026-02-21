use itertools::Itertools;
use tombi_syntax::SyntaxNode;

use crate::{AstNode, support};
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
    pub fn values_with_comma(&self) -> impl Iterator<Item = (crate::Value, Option<crate::Comma>)> {
        self.values()
            .zip_longest(support::node::children::<crate::Comma>(self.syntax()))
            .filter_map(|value_with_comma| match value_with_comma {
                itertools::EitherOrBoth::Both(value, comma) => Some((value, Some(comma))),
                itertools::EitherOrBoth::Left(value) => Some((value, None)),
                itertools::EitherOrBoth::Right(_) => None,
            })
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
