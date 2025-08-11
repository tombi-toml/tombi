use std::{fmt, marker::PhantomData};

use crate::{
    cursor,
    green::{GreenNode, GreenToken, GreenTokenData},
    red::{RedElement, RedNode},
    Direction, Language, NodeOrToken,
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct RedToken<L: Language> {
    raw: cursor::SyntaxToken,
    _p: PhantomData<L>,
}

impl<L: Language> fmt::Debug for RedToken<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} @{:?} @{:?}",
            self.kind(),
            self.span(),
            self.range()
        )?;
        if self.text().len() < 25 {
            return write!(f, " {:?}", self.text());
        }
        let text = self.text();
        for idx in 21..25 {
            if text.is_char_boundary(idx) {
                let text = format!("{} ...", &text[..idx]);
                return write!(f, " {text:?}");
            }
        }
        unreachable!()
    }
}

impl<L: Language> fmt::Display for RedToken<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.raw, f)
    }
}

impl<L: Language> RedToken<L> {
    /// Returns a green tree, equal to the green tree this token
    /// belongs to, except with this token substituted. The complexity
    /// of the operation is proportional to the depth of the tree.
    pub fn replace_with(&self, new_token: GreenToken) -> GreenNode {
        self.raw.replace_with(new_token)
    }

    pub fn kind(&self) -> L::Kind {
        L::kind_from_raw(self.raw.kind())
    }

    pub fn span(&self) -> tombi_text::Span {
        self.raw.span()
    }

    pub fn range(&self) -> tombi_text::Range {
        self.raw.range()
    }

    pub fn index(&self) -> usize {
        self.raw.index()
    }

    pub fn text(&self) -> &str {
        self.raw.text()
    }

    pub fn green(&self) -> &GreenTokenData {
        self.raw.green()
    }

    pub fn parent(&self) -> Option<RedNode<L>> {
        self.raw.parent().map(RedNode::from)
    }

    /// Iterator over all the ancestors of this token excluding itself.
    #[deprecated = "use `SyntaxToken::parent_ancestors` instead"]
    pub fn ancestors(&self) -> impl Iterator<Item = RedNode<L>> {
        self.parent_ancestors()
    }

    /// Iterator over all the ancestors of this token excluding itself.
    pub fn parent_ancestors(&self) -> impl Iterator<Item = RedNode<L>> {
        self.raw.ancestors().map(RedNode::from)
    }

    pub fn next_sibling_or_token(&self) -> Option<RedElement<L>> {
        self.raw.next_sibling_or_token().map(NodeOrToken::from)
    }
    pub fn prev_sibling_or_token(&self) -> Option<RedElement<L>> {
        self.raw.prev_sibling_or_token().map(NodeOrToken::from)
    }

    pub fn siblings_with_tokens(
        &self,
        direction: Direction,
    ) -> impl Iterator<Item = RedElement<L>> {
        self.raw
            .siblings_with_tokens(direction)
            .map(RedElement::from)
    }

    /// Next token in the tree (i.e, not necessary a sibling).
    pub fn next_token(&self) -> Option<RedToken<L>> {
        self.raw.next_token().map(RedToken::from)
    }
    /// Previous token in the tree (i.e, not necessary a sibling).
    pub fn prev_token(&self) -> Option<RedToken<L>> {
        self.raw.prev_token().map(RedToken::from)
    }

    pub fn detach(&self) {
        self.raw.detach()
    }
}

impl<L: Language> From<cursor::SyntaxToken> for RedToken<L> {
    fn from(raw: cursor::SyntaxToken) -> RedToken<L> {
        RedToken {
            raw,
            _p: PhantomData,
        }
    }
}

impl<L: Language> From<RedToken<L>> for cursor::SyntaxToken {
    fn from(token: RedToken<L>) -> cursor::SyntaxToken {
        token.raw
    }
}
