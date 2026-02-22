use tombi_syntax::{SyntaxKind::*, T};

use crate::{
    ErrorKind::*, parse::Parse, parser::Parser, support::peek_leading_comments,
    token_set::TS_KEY_FIRST,
};

impl Parse for tombi_ast::KeyValueWithCommaGroup {
    fn parse(p: &mut Parser<'_>) {
        let m = p.start();

        loop {
            tombi_ast::KeyValue::parse(p);
            maybe_comma(p);

            if !p.at(LINE_BREAK) {
                break;
            }

            if p.nth_at(1, LINE_BREAK) {
                break;
            }

            let n = peek_leading_comments(p);
            if !p.nth_at_ts(n, TS_KEY_FIRST) {
                break;
            }

            while p.eat(LINE_BREAK) {}
        }

        m.complete(p, KEY_VALUE_WITH_COMMA_GROUP);
    }
}

fn maybe_comma(p: &mut Parser<'_>) {
    let n = peek_leading_comments(p);
    if p.nth_at(n, T![,]) {
        tombi_ast::Comma::parse(p);
    } else if !p.nth_at(n, T!['}']) {
        p.error(crate::Error::new(ExpectedComma, p.current_range()));
        p.bump_any();
    }
}
