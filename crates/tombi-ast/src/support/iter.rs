use crate::AstNode;

pub(crate) struct WithCommaIter<T, I> {
    nodes: I,
    pending_item: Option<T>,
}

impl<T, I> WithCommaIter<T, I> {
    pub(crate) fn new(nodes: I) -> Self {
        Self {
            nodes,
            pending_item: None,
        }
    }
}

impl<T, I> Iterator for WithCommaIter<T, I>
where
    T: AstNode,
    I: Iterator<Item = tombi_syntax::SyntaxNode>,
{
    type Item = (T, Option<crate::Comma>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some(node) = self.nodes.next() else {
                return self.pending_item.take().map(|item| (item, None));
            };

            if T::can_cast(node.kind()) {
                let item = T::cast(node).unwrap();
                if let Some(prev) = self.pending_item.replace(item) {
                    return Some((prev, None));
                }
                continue;
            }

            if crate::Comma::can_cast(node.kind()) {
                let comma = crate::Comma::cast(node).unwrap();
                if let Some(prev) = self.pending_item.take() {
                    return Some((prev, Some(comma)));
                }
            }
        }
    }
}
