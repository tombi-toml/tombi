use std::fmt;

use crate::cursor::{SyntaxNode, SyntaxToken};

#[derive(Clone)]
pub struct SyntaxText {
    node: SyntaxNode,
    span: tombi_text::Span,
}

impl SyntaxText {
    #[inline]
    pub(crate) fn new(node: SyntaxNode) -> SyntaxText {
        let span = node.span();
        SyntaxText { node, span }
    }

    pub fn len(&self) -> tombi_text::RawOffset {
        self.span.len()
    }

    pub fn is_empty(&self) -> bool {
        self.span.is_empty()
    }

    pub fn contains_char(&self, c: char) -> bool {
        self.try_for_each_chunk(|chunk| if chunk.contains(c) { Err(()) } else { Ok(()) })
            .is_err()
    }

    pub fn find_char(&self, c: char) -> Option<tombi_text::Offset> {
        let mut acc: tombi_text::Offset = 0.into();
        let res = self.try_for_each_chunk(|chunk| {
            if let Some(pos) = chunk.find(c) {
                return Err(acc + pos as tombi_text::RelativeOffset);
            }
            acc += tombi_text::Offset::of(chunk);
            Ok(())
        });
        found(res)
    }

    pub fn char_at(&self, offset: tombi_text::Offset) -> Option<char> {
        let mut start: tombi_text::Offset = 0.into();
        let res = self.try_for_each_chunk(|chunk| {
            let end = start + tombi_text::Offset::of(chunk);
            if start <= offset && offset < end {
                let off: usize = u32::from(offset - start) as usize;
                return Err(chunk[off..].chars().next().unwrap());
            }
            start = end;
            Ok(())
        });
        found(res)
    }

    pub fn slice<S: private::SyntaxTextSpan>(&self, span: S) -> SyntaxText {
        let start = span.start().unwrap_or_default();
        let end = span.end().unwrap_or(tombi_text::Offset::new(self.len()));
        assert!(start <= end);
        let len = end - start;
        let start = self.span.start + start;
        let end = start + len;
        let span = tombi_text::Span::new(start, end);

        SyntaxText {
            node: self.node.clone(),
            span,
        }
    }

    pub fn try_fold_chunks<T, F, E>(&self, init: T, mut f: F) -> Result<T, E>
    where
        F: FnMut(T, &str) -> Result<T, E>,
    {
        self.tokens_with_spans()
            .try_fold(init, move |acc, (token, span)| f(acc, &token.text()[span]))
    }

    pub fn try_for_each_chunk<F: FnMut(&str) -> Result<(), E>, E>(
        &self,
        mut f: F,
    ) -> Result<(), E> {
        self.try_fold_chunks((), move |(), chunk| f(chunk))
    }

    pub fn for_each_chunk<F: FnMut(&str)>(&self, mut f: F) {
        enum Void {}
        match self.try_for_each_chunk(|chunk| {
            f(chunk);
            Ok::<(), Void>(())
        }) {
            Ok(()) => (),
            Err(void) => match void {},
        }
    }

    fn tokens_with_spans(&self) -> impl Iterator<Item = (SyntaxToken, tombi_text::Span)> {
        let span = self.span;
        self.node
            .descendants_with_tokens()
            .filter_map(|element| element.into_token())
            .filter_map(move |token| {
                let token_span = token.span();
                let span = span.intersect(token_span)?;
                Some((token, span - token_span.start))
            })
    }
}

fn found<T>(res: Result<(), T>) -> Option<T> {
    res.err()
}

impl fmt::Debug for SyntaxText {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.to_string(), f)
    }
}

impl fmt::Display for SyntaxText {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.try_for_each_chunk(|chunk| fmt::Display::fmt(chunk, f))
    }
}

impl From<SyntaxText> for String {
    fn from(text: SyntaxText) -> String {
        text.to_string()
    }
}

impl PartialEq<str> for SyntaxText {
    fn eq(&self, mut rhs: &str) -> bool {
        self.try_for_each_chunk(|chunk| {
            if !rhs.starts_with(chunk) {
                return Err(());
            }
            rhs = &rhs[chunk.len()..];
            Ok(())
        })
        .is_ok()
            && rhs.is_empty()
    }
}

impl PartialEq<SyntaxText> for str {
    fn eq(&self, rhs: &SyntaxText) -> bool {
        rhs == self
    }
}

impl PartialEq<&'_ str> for SyntaxText {
    fn eq(&self, rhs: &&str) -> bool {
        self == *rhs
    }
}

impl PartialEq<SyntaxText> for &'_ str {
    fn eq(&self, rhs: &SyntaxText) -> bool {
        rhs == self
    }
}

impl PartialEq for SyntaxText {
    fn eq(&self, other: &SyntaxText) -> bool {
        if self.span.len() != other.span.len() {
            return false;
        }
        let mut lhs = self.tokens_with_spans();
        let mut rhs = other.tokens_with_spans();
        zip_texts(&mut lhs, &mut rhs).is_none()
            && lhs.all(|it| it.1.is_empty())
            && rhs.all(|it| it.1.is_empty())
    }
}

fn zip_texts<I: Iterator<Item = (SyntaxToken, tombi_text::Span)>>(
    xs: &mut I,
    ys: &mut I,
) -> Option<()> {
    let mut x = xs.next()?;
    let mut y = ys.next()?;
    loop {
        while x.1.is_empty() {
            x = xs.next()?;
        }
        while y.1.is_empty() {
            y = ys.next()?;
        }
        let x_text = &x.0.text()[x.1];
        let y_text = &y.0.text()[y.1];
        if !(x_text.starts_with(y_text) || y_text.starts_with(x_text)) {
            return Some(());
        }
        let advance = tombi_text::Offset::new(std::cmp::min(x.1.len(), y.1.len()));
        x.1 = tombi_text::Span::new(x.1.start + advance, x.1.end);
        y.1 = tombi_text::Span::new(y.1.start + advance, y.1.end);
    }
}

impl Eq for SyntaxText {}

mod private {
    use std::ops;

    pub trait SyntaxTextSpan {
        fn start(&self) -> Option<tombi_text::Offset>;
        fn end(&self) -> Option<tombi_text::Offset>;
    }

    impl SyntaxTextSpan for tombi_text::Span {
        fn start(&self) -> Option<tombi_text::Offset> {
            Some(self.start)
        }
        fn end(&self) -> Option<tombi_text::Offset> {
            Some(self.end)
        }
    }

    impl SyntaxTextSpan for ops::Range<tombi_text::Offset> {
        fn start(&self) -> Option<tombi_text::Offset> {
            Some(self.start)
        }
        fn end(&self) -> Option<tombi_text::Offset> {
            Some(self.end)
        }
    }

    impl SyntaxTextSpan for ops::RangeFrom<tombi_text::Offset> {
        fn start(&self) -> Option<tombi_text::Offset> {
            Some(self.start)
        }
        fn end(&self) -> Option<tombi_text::Offset> {
            None
        }
    }

    impl SyntaxTextSpan for ops::RangeTo<tombi_text::Offset> {
        fn start(&self) -> Option<tombi_text::Offset> {
            None
        }
        fn end(&self) -> Option<tombi_text::Offset> {
            Some(self.end)
        }
    }

    impl SyntaxTextSpan for ops::RangeFull {
        fn start(&self) -> Option<tombi_text::Offset> {
            None
        }
        fn end(&self) -> Option<tombi_text::Offset> {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{green::SyntaxKind, GreenNodeBuilder};

    fn build_tree(chunks: &[&str]) -> SyntaxNode {
        let mut builder = GreenNodeBuilder::new();
        builder.start_node(SyntaxKind(62));
        for &chunk in chunks.iter() {
            builder.token(SyntaxKind(92), chunk)
        }
        builder.finish_node();
        SyntaxNode::new_root(builder.finish())
    }

    #[test]
    fn test_text_equality() {
        fn do_check(t1: &[&str], t2: &[&str]) {
            let t1 = build_tree(t1).text();
            let t2 = build_tree(t2).text();
            let expected = t1.to_string() == t2.to_string();
            let actual = t1 == t2;
            pretty_assertions::assert_eq!(
                expected,
                actual,
                "`{}` (SyntaxText) `{}` (SyntaxText)",
                t1,
                t2
            );
            let actual = t1 == *t2.to_string();
            pretty_assertions::assert_eq!(
                expected,
                actual,
                "`{}` (SyntaxText) `{}` (&str)",
                t1,
                t2
            );
        }
        fn check(t1: &[&str], t2: &[&str]) {
            do_check(t1, t2);
            do_check(t2, t1)
        }

        check(&[""], &[""]);
        check(&["a"], &[""]);
        check(&["a"], &["a"]);
        check(&["abc"], &["def"]);
        check(&["hello", "world"], &["hello", "world"]);
        check(&["hellowo", "rld"], &["hell", "oworld"]);
        check(&["hel", "lowo", "rld"], &["helloworld"]);
        check(&["{", "abc", "}"], &["{", "123", "}"]);
        check(&["{", "abc", "}", "{"], &["{", "123", "}"]);
        check(&["{", "abc", "}"], &["{", "123", "}", "{"]);
        check(&["{", "abc", "}ab"], &["{", "abc", "}", "ab"]);
    }
}
