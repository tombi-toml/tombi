use std::marker::PhantomData;

use crate::{RedElement, RedNode, WalkEvent, cursor, language::Language};

pub struct Preorder<L: Language> {
    raw: cursor::Preorder,
    _p: PhantomData<L>,
}

impl<L: Language> Preorder<L> {
    pub fn skip_subtree(&mut self) {
        self.raw.skip_subtree()
    }
}

impl<L: Language> Iterator for Preorder<L> {
    type Item = WalkEvent<RedNode<L>>;
    fn next(&mut self) -> Option<Self::Item> {
        self.raw.next().map(|it| it.map(RedNode::from))
    }
}

impl<L: Language> From<cursor::Preorder> for Preorder<L> {
    fn from(raw: cursor::Preorder) -> Self {
        Self {
            raw,
            _p: PhantomData,
        }
    }
}

pub struct RedPreorderWithTokens<L: Language> {
    raw: cursor::PreorderWithTokens,
    _p: PhantomData<L>,
}

impl<L: Language> RedPreorderWithTokens<L> {
    pub fn skip_subtree(&mut self) {
        self.raw.skip_subtree()
    }
}

impl<L: Language> Iterator for RedPreorderWithTokens<L> {
    type Item = WalkEvent<RedElement<L>>;
    fn next(&mut self) -> Option<Self::Item> {
        self.raw.next().map(|it| it.map(RedElement::from))
    }
}

impl<L: Language> From<cursor::PreorderWithTokens> for RedPreorderWithTokens<L> {
    fn from(raw: cursor::PreorderWithTokens) -> Self {
        Self {
            raw,
            _p: PhantomData,
        }
    }
}
