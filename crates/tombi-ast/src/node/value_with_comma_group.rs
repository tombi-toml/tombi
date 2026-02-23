use tombi_syntax::SyntaxNode;

use crate::AstNode;
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
        ValuesWithCommaIter::new(self.syntax().children())
    }

    #[inline]
    pub fn into_values_with_comma(
        self,
    ) -> impl Iterator<Item = (crate::Value, Option<crate::Comma>)> {
        ValuesWithCommaIter::new(self.syntax.children())
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

struct ValuesWithCommaIter<I> {
    nodes: I,
    pending_value: Option<crate::Value>,
}

impl<I> ValuesWithCommaIter<I> {
    fn new(nodes: I) -> Self {
        Self {
            nodes,
            pending_value: None,
        }
    }
}

impl<I> Iterator for ValuesWithCommaIter<I>
where
    I: Iterator<Item = tombi_syntax::SyntaxNode>,
{
    type Item = (crate::Value, Option<crate::Comma>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some(node) = self.nodes.next() else {
                return self.pending_value.take().map(|value| (value, None));
            };

            if crate::Value::can_cast(node.kind()) {
                let value = crate::Value::cast(node).unwrap();
                if let Some(prev) = self.pending_value.replace(value) {
                    return Some((prev, None));
                }
                continue;
            }

            if crate::Comma::can_cast(node.kind()) {
                let comma = crate::Comma::cast(node).unwrap();
                if let Some(prev) = self.pending_value.take() {
                    return Some((prev, Some(comma)));
                }
            }
        }
    }
}
