use tombi_syntax::{SyntaxKind::*, T};

use crate::{
    parse::{Parse, is_group_separator},
    parser::Parser,
    support::peek_leading_comments,
    token_set::{TS_INLINE_TABLE_END, TS_KEY_FIRST},
};

impl Parse for tombi_ast::KeyValueWithCommaGroup {
    fn parse(p: &mut Parser<'_>) {
        let m = p.start();

        loop {
            if is_group_separator(p) {
                break;
            }

            tombi_ast::KeyValue::parse(p);

            let n = peek_leading_comments(p);
            if p.nth_at(n, T![,]) {
                tombi_ast::Comma::parse(p);
            } else if p.nth_at_ts(n, TS_INLINE_TABLE_END) {
                break;
            }

            let n = peek_leading_comments(p);
            if !p.nth_at_ts(n, TS_KEY_FIRST) {
                break;
            }
        }

        m.complete(p, KEY_VALUE_WITH_COMMA_GROUP);
    }
}
