use crate::{SyntaxKind::*, token_set::TS_LEADING_COMMENTS_KINDS};

pub fn peek_leading_comments(p: &mut crate::parser::Parser<'_>) -> usize {
    let mut n = 0;
    loop {
        if p.nth_at(n, LINE_BREAK) {
            n += 1;
        } else if p.nth_at(n, COMMENT)
            && p.nth_at(n + 1, LINE_BREAK)
            && !p.nth_at(n + 2, LINE_BREAK)
        {
            n += 2;
        } else {
            return n;
        }
    }
}

#[inline]
pub fn leading_comments(p: &mut crate::parser::Parser<'_>) {
    while p.eat_ts(TS_LEADING_COMMENTS_KINDS) {}
}

#[inline]
pub fn trailing_comment(p: &mut crate::parser::Parser<'_>) {
    while p.eat(COMMENT) {}
}
