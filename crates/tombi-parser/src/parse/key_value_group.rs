use tombi_syntax::SyntaxKind::*;

use crate::{
    parse::Parse,
    parser::Parser,
    support::peek_leading_comments,
    token_set::{TS_KEY_FIRST, TS_NEXT_SECTION},
};

impl Parse for tombi_ast::KeyValueGroup {
    fn parse(p: &mut Parser<'_>) {
        let m = p.start();

        tombi_ast::KeyValue::parse(p);

        loop {
            if !p.eat(LINE_BREAK) {
                break;
            }
            let n = peek_leading_comments(p);
            if p.nth_at_ts(n, TS_NEXT_SECTION) {
                break;
            }
            if !p.nth_at_ts(n, TS_KEY_FIRST) {
                break;
            }
            tombi_ast::KeyValue::parse(p);
        }

        m.complete(p, KEY_VALUE_GROUP);
    }
}
