use std::marker::PhantomData;

use crate::{Language, cursor, red::RedNode};

#[derive(Debug, Clone)]
pub struct RedNodeChildren<L: Language> {
    raw: cursor::SyntaxNodeChildren,
    _p: PhantomData<L>,
}

impl<L: Language> Iterator for RedNodeChildren<L> {
    type Item = RedNode<L>;
    fn next(&mut self) -> Option<Self::Item> {
        self.raw.next().map(RedNode::from)
    }
}

impl<L: Language> From<cursor::SyntaxNodeChildren> for RedNodeChildren<L> {
    fn from(raw: cursor::SyntaxNodeChildren) -> Self {
        Self {
            raw,
            _p: PhantomData,
        }
    }
}
