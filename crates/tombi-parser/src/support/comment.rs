use crate::{
    SyntaxKind::*,
    token_set::{TS_DANGLING_COMMENTS_KINDS, TS_LEADING_COMMENTS_KINDS},
};

pub fn begin_dangling_comments(p: &mut crate::parser::Parser<'_>) {
    dangling_comments(p);
}

pub fn end_dangling_comments(p: &mut crate::parser::Parser<'_>, last_eat: bool) {
    if last_eat {
        while p.eat_ts(TS_DANGLING_COMMENTS_KINDS) {}
    } else {
        dangling_comments(p);
    }
}

fn dangling_comments(p: &mut crate::parser::Parser<'_>) {
    while p.eat(LINE_BREAK) {}

    let mut n = 0;
    let mut comment_count = 0;
    while p.nth_at_ts(n, TS_DANGLING_COMMENTS_KINDS) {
        let kind = p.nth(n);
        match kind {
            COMMENT => {
                comment_count += 1;
            }
            LINE_BREAK => {
                while p.nth_at(n + 1, WHITESPACE) {
                    n += 1;
                }
                if p.nth_at(n + 1, LINE_BREAK) {
                    if comment_count > 0 {
                        (0..=n).for_each(|_| p.bump_any());
                        while p.eat(LINE_BREAK) || p.eat(WHITESPACE) {}
                        if p.at(COMMENT) {
                            n = 0;
                            comment_count = 0;
                            continue;
                        }
                        break;
                    }
                    n += 1;
                }
            }
            _ => unreachable!("unexpected token {:?}", kind),
        }
        n += 1;
    }

    if p.nth_at(n + 1, EOF) {
        for _ in 0..=n {
            if !p.eat_ts(TS_DANGLING_COMMENTS_KINDS) {
                break;
            }
        }
    }
}

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
