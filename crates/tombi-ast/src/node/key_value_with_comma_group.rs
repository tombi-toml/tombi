use tombi_syntax::SyntaxNode;

use crate::AstNode;
use tombi_syntax::SyntaxKind::KEY_VALUE_WITH_COMMA_GROUP;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyValueWithCommaGroup {
    pub(crate) syntax: SyntaxNode,
}

impl KeyValueWithCommaGroup {
    #[inline]
    pub fn key_values(&self) -> impl Iterator<Item = crate::KeyValue> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|el| el.into_node().and_then(crate::KeyValue::cast))
    }

    #[inline]
    pub fn into_key_values(self) -> impl Iterator<Item = crate::KeyValue> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|el| el.into_node().and_then(crate::KeyValue::cast))
    }

    #[inline]
    pub fn key_values_with_comma(
        &self,
    ) -> impl Iterator<Item = (crate::KeyValue, Option<crate::Comma>)> {
        KeyValuesWithCommaIter::new(self.syntax().children())
    }

    #[inline]
    pub fn into_key_values_with_comma(
        self,
    ) -> impl Iterator<Item = (crate::KeyValue, Option<crate::Comma>)> {
        KeyValuesWithCommaIter::new(self.syntax.children())
    }

    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.syntax.range()
    }
}

impl AstNode for KeyValueWithCommaGroup {
    #[inline]
    fn can_cast(kind: tombi_syntax::SyntaxKind) -> bool {
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

struct KeyValuesWithCommaIter<I> {
    nodes: I,
    pending_key_value: Option<crate::KeyValue>,
}

impl<I> KeyValuesWithCommaIter<I> {
    fn new(elements: I) -> Self {
        Self {
            nodes: elements,
            pending_key_value: None,
        }
    }
}

impl<I> Iterator for KeyValuesWithCommaIter<I>
where
    I: Iterator<Item = tombi_syntax::SyntaxNode>,
{
    type Item = (crate::KeyValue, Option<crate::Comma>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some(node) = self.nodes.next() else {
                return self
                    .pending_key_value
                    .take()
                    .map(|key_value| (key_value, None));
            };

            if crate::KeyValue::can_cast(node.kind()) {
                let key_value = crate::KeyValue::cast(node).unwrap();
                if let Some(prev) = self.pending_key_value.replace(key_value) {
                    return Some((prev, None));
                }
                continue;
            }

            if crate::Comma::can_cast(node.kind()) {
                let comma = crate::Comma::cast(node).unwrap();
                if let Some(prev) = self.pending_key_value.take() {
                    return Some((prev, Some(comma)));
                }
            }
        }
    }
}
