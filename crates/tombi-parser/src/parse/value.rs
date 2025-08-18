use tombi_syntax::{SyntaxKind::*, T};

use super::{key::eat_key, leading_comments, peek_leading_comments, trailing_comment, Parse};
use crate::{parser::Parser, token_set::TS_COMMEMT_OR_LINE_END, ErrorKind::*};

impl Parse for tombi_ast::Value {
    fn parse(p: &mut Parser<'_>) {
        let n = peek_leading_comments(p);
        match p.nth(n) {
            BASIC_STRING
            | MULTI_LINE_BASIC_STRING
            | LITERAL_STRING
            | MULTI_LINE_LITERAL_STRING
            | INTEGER_DEC
            | INTEGER_BIN
            | INTEGER_OCT
            | INTEGER_HEX
            | FLOAT
            | BOOLEAN
            | OFFSET_DATE_TIME
            | LOCAL_DATE_TIME
            | LOCAL_DATE
            | LOCAL_TIME => parse_literal_value(p),
            T!('[') => tombi_ast::Array::parse(p),
            T!('{') => tombi_ast::InlineTable::parse(p),
            BARE_KEY => {
                // NOTE: This is a hack to make code completion more comfortable.

                let key_range = p.nth_range(n);
                p.error(crate::Error::new(ExpectedValue, key_range));
                let m = p.start();
                leading_comments(p);
                {
                    let m = p.start();
                    if eat_key(p) {
                        m.complete(p, KEYS);
                    }
                }
                trailing_comment(p);
                m.complete(p, KEY_VALUE);
            }
            _ => parse_invalid_value(p, n),
        }
    }
}

fn parse_literal_value(p: &mut Parser<'_>) {
    let m = p.start();

    leading_comments(p);

    let kind = p.current();

    p.bump(kind);

    trailing_comment(p);

    m.complete(p, kind);
}

fn parse_invalid_value(p: &mut Parser<'_>, n: usize) {
    let m = p.start();

    if n > 1 && p.nth_at(n, LINE_BREAK) {
        leading_comments(p);
    }

    let start_range = p.current_range();
    let mut end_range = start_range;
    while !p.at_ts(TS_COMMEMT_OR_LINE_END) {
        end_range = p.current_range();
        p.bump_any();
    }
    p.error(crate::Error::new(ExpectedValue, start_range + end_range));

    trailing_comment(p);

    m.complete(p, INVALID_TOKEN);
}
