use tombi_syntax::SyntaxKind::*;

use crate::{
    parse::{Parse, is_group_separator},
    parser::Parser,
    support::peek_leading_comments,
    token_set::TS_KEY_FIRST,
};

impl Parse for tombi_ast::KeyValueGroup {
    fn parse(p: &mut Parser<'_>) {
        let m = p.start();

        loop {
            if is_group_separator(p) {
                break;
            }

            tombi_ast::KeyValue::parse(p);

            if !p.at(LINE_BREAK) {
                break;
            }

            let n = peek_leading_comments(p);
            if !p.nth_at_ts(n, TS_KEY_FIRST) {
                break;
            }
        }

        m.complete(p, KEY_VALUE_GROUP);
    }
}
