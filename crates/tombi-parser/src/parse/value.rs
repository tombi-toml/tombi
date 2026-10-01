use tombi_ast_syntax::{SyntaxKind::*, T};

use super::Parse;
use crate::parse::key::eat_keys;
use crate::support::{leading_comments, peek_leading_comments, trailing_comment};
use crate::{
    ErrorKind::*,
    parser::{MAX_RECURSION_DEPTH, Parser},
    token_set::TS_COMMEMT_OR_LINE_END,
};

impl Parse for tombi_ast_syntax::Value<'_> {
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
            T!('[') => parse_nested(p, <tombi_ast_syntax::Array as Parse>::parse),
            T!('{') => parse_nested(p, <tombi_ast_syntax::InlineTable as Parse>::parse),
            BARE_KEY => {
                // NOTE: This is a hack to make code completion more comfortable.

                let key_span = p.nth_span(n);
                p.error(crate::Error::new(ExpectedValue, key_span));
                let m = p.start();
                leading_comments(p);
                {
                    let m = p.start();
                    if eat_keys(p) {
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

/// Parse a nested array/inline table, bounding recursion depth.
///
/// Past [`MAX_RECURSION_DEPTH`] the nested value is skipped as an invalid token
/// instead of recursing, so a deeply nested document cannot overflow the stack.
fn parse_nested(p: &mut Parser<'_>, parse_inner: fn(&mut Parser<'_>)) {
    if p.nested_depth() >= MAX_RECURSION_DEPTH {
        skip_too_deep_value(p);
    } else {
        p.enter_nested();
        parse_inner(p);
        p.exit_nested();
    }
}

/// Consume an over-nested bracketed region without recursing.
///
/// Iteratively skips the balanced `[...]` / `{...}` span (or up to EOF for an
/// unterminated one) and reports a single `RecursionLimitExceeded` error.
fn skip_too_deep_value(p: &mut Parser<'_>) {
    let m = p.start();

    leading_comments(p);

    let start_span = p.current_span();
    let mut end_span = start_span;
    let mut bracket_depth: usize = 0;
    loop {
        match p.current() {
            EOF => break,
            T!('[') | T!('{') => {
                bracket_depth += 1;
                end_span = p.current_span();
                p.bump_any();
            }
            T!(']') | T!('}') => {
                end_span = p.current_span();
                p.bump_any();
                bracket_depth -= 1;
                if bracket_depth == 0 {
                    break;
                }
            }
            _ => {
                end_span = p.current_span();
                p.bump_any();
            }
        }
    }

    p.error(crate::Error::new(
        RecursionLimitExceeded,
        start_span + end_span,
    ));

    trailing_comment(p);

    m.complete(p, INVALID_TOKEN);
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

    let start_span = p.current_span();
    let mut end_span = start_span;
    while !p.at_ts(TS_COMMEMT_OR_LINE_END) {
        end_span = p.current_span();
        p.bump_any();
    }
    p.error(crate::Error::new(ExpectedValue, start_span + end_span));

    trailing_comment(p);

    m.complete(p, INVALID_TOKEN);
}

#[cfg(test)]
mod test {
    use crate::{ErrorKind::RecursionLimitExceeded, test_parser};

    test_parser! {
        #[test]
        fn nesting_within_limit(
            format!("key = {}1{}", "[".repeat(128), "]".repeat(128))
        ) -> Assert(|parsed| {
            parsed.errors.is_empty()
        })
    }

    test_parser! {
        #[test]
        fn nesting_beyond_limit(
            format!("key = {}1{}", "[".repeat(129), "]".repeat(129))
        ) -> Assert(|parsed| {
            parsed
                .errors
                .iter()
                .any(|error| error.kind() == RecursionLimitExceeded)
        })
    }

    test_parser! {
        #[test]
        fn deeply_nested_arrays_do_not_overflow_stack(
            format!("key = {}", "[".repeat(500_000))
        ) -> Assert(|parsed| {
            let has_errors = !parsed.errors.is_empty();
            drop(parsed);
            has_errors
        })
    }

    test_parser! {
        #[test]
        fn deeply_nested_inline_tables_do_not_overflow_stack(
            format!("key = {}", "{a=".repeat(300_000))
        ) -> Assert(|parsed| {
            let has_errors = !parsed.errors.is_empty();
            drop(parsed);
            has_errors
        })
    }
}
