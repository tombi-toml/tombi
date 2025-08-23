mod array;
mod array_of_table;
mod comma;
mod inline_table;
mod key;
mod key_value;
mod root;
mod table;
mod value;

use support::*;

use crate::{parser::Parser, token_set::TS_LINE_END};

pub(crate) trait Parse {
    fn parse(p: &mut Parser<'_>);
}

pub fn invalid_line(p: &mut Parser<'_>, kind: crate::ErrorKind) {
    p.error(crate::Error::new(kind, p.current_range()));
    p.bump_any();
    while !p.at_ts(TS_LINE_END) {
        p.bump_any();
    }
}

mod support {
    use crate::{
        token_set::{
            TS_DANGLING_COMMENTS_KINDS, TS_LEADING_COMMENTS_KINDS, TS_TAILING_COMMENT_KINDS,
        },
        SyntaxKind::*,
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
        while p.nth_at_ts(n, TS_LEADING_COMMENTS_KINDS) {
            n += 1;
        }

        n
    }

    #[inline]
    pub fn leading_comments(p: &mut crate::parser::Parser<'_>) {
        while p.eat_ts(TS_LEADING_COMMENTS_KINDS) {}
    }

    #[inline]
    pub fn trailing_comment(p: &mut crate::parser::Parser<'_>) {
        while p.eat_ts(TS_TAILING_COMMENT_KINDS) {}
    }
}
