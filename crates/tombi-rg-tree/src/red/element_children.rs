use std::marker::PhantomData;

use crate::{NodeOrToken, RedElement, cursor, language::Language};

#[derive(Debug, Clone)]
pub struct RedElementChildren<L: Language> {
    raw: cursor::SyntaxElementChildren,
    _p: PhantomData<L>,
}

impl<L: Language> Iterator for RedElementChildren<L> {
    type Item = RedElement<L>;
    fn next(&mut self) -> Option<Self::Item> {
        self.raw.next().map(NodeOrToken::from)
    }
}

impl<L: Language> From<cursor::SyntaxElementChildren> for RedElementChildren<L> {
    fn from(raw: cursor::SyntaxElementChildren) -> Self {
        Self {
            raw,
            _p: PhantomData,
        }
    }
}
