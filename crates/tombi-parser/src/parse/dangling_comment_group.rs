use tombi_syntax::SyntaxKind::*;

use crate::{parse::Parse, parser::Parser, token_set::TS_NEXT_SECTION};

impl Parse for Vec<tombi_ast::DanglingCommentGroup> {
    fn parse(p: &mut Parser<'_>) {
        loop {
            while p.eat(LINE_BREAK) {}

            let Some(group_len) = dangling_comment_group_len(p) else {
                break;
            };

            let m = p.start();
            (0..group_len).for_each(|_| p.bump_any());
            m.complete(p, DANGLING_COMMENT_GROUP);
        }
    }
}

fn dangling_comment_group_len(p: &Parser<'_>) -> Option<usize> {
    if !p.nth_at(0, COMMENT) {
        return None;
    }

    let mut n = 0;
    loop {
        debug_assert!(p.nth_at(n, COMMENT));
        n += 1;

        if p.nth_at(n, EOF) {
            return Some(n);
        }

        if !p.nth_at(n, LINE_BREAK) {
            return None;
        }

        if p.nth_at(n + 1, COMMENT) {
            n += 1;
            continue;
        }

        if p.nth_at_ts(n, TS_NEXT_SECTION) {
            return Some(n);
        }

        if p.nth_at(n + 1, LINE_BREAK) || p.nth_at(n + 1, EOF) {
            return Some(n);
        }

        // Closing brackets can't have leading comments,
        // so comments before them are dangling.
        if p.nth_at(n + 1, BRACE_END) || p.nth_at(n + 1, BRACKET_END) {
            return Some(n);
        }

        return None;
    }
}
