use tombi_syntax::{SyntaxKind::*, T};

use crate::{
    ErrorKind::*,
    parse::{Parse, is_group_separator},
    parser::Parser,
    support::peek_leading_comments,
    token_set::TS_VALUE_FIRST,
};

impl Parse for tombi_ast::ValueWithCommaGroup {
    fn parse(p: &mut Parser<'_>) {
        let m = p.start();

        loop {
            if is_group_separator(p) {
                break;
            }

            tombi_ast::Value::parse(p);

            let n = peek_leading_comments(p);
            if p.nth_at(n, T![,]) {
                tombi_ast::Comma::parse(p);
            } else if !p.nth_at(n, T![']']) {
                p.error(crate::Error::new(ExpectedComma, p.current_range()));
                p.bump_any();
            }

            let n = peek_leading_comments(p);
            if !p.nth_at_ts(n, TS_VALUE_FIRST) {
                break;
            }
        }

        m.complete(p, VALUE_WITH_COMMA_GROUP);
    }
}
