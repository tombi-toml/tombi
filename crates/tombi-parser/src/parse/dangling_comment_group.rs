use tombi_syntax::SyntaxKind::*;

use crate::{
    parse::Parse,
    parser::Parser,
    token_set::{TS_DANGLING_COMMENT_GROUP_END, TS_LINE_END},
};

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
    let mut n = 0;
    if !p.nth_at(n, COMMENT) {
        return None;
    }
    loop {
        if !p.nth_at_ts(n + 1, TS_LINE_END) {
            return None;
        }
        if p.nth_at_ts(n + 2, TS_DANGLING_COMMENT_GROUP_END) {
            return Some(n + 1);
        }
        if !p.nth_at(n + 2, COMMENT) {
            return None;
        }
        n += 2;
    }
}
