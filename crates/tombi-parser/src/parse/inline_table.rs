use tombi_syntax::{SyntaxKind::*, T};

use crate::{
    ErrorKind::*,
    parse::{
        Parse, begin_dangling_comments, end_dangling_comments, leading_comments,
        peek_leading_comments, trailing_comment,
    },
    parser::Parser,
};

impl Parse for tombi_ast::InlineTable {
    fn parse(p: &mut Parser<'_>) {
        let m = p.start();

        leading_comments(p);

        debug_assert!(p.at(T!['{']));

        p.eat(T!['{']);

        trailing_comment(p);

        begin_dangling_comments(p);

        loop {
            while p.eat(LINE_BREAK) {}

            let n = peek_leading_comments(p);
            if p.nth_at(n, EOF) || p.nth_at(n, T!['}']) {
                break;
            }

            tombi_ast::KeyValue::parse(p);

            let n = peek_leading_comments(p);
            if p.nth_at(n, T![,]) {
                tombi_ast::Comma::parse(p);
            } else {
                if !p.nth_at(n, T!['}']) {
                    p.error(crate::Error::new(ExpectedComma, p.current_range()));
                    p.bump_any();
                }
            }
        }

        end_dangling_comments(p, true);

        if !p.eat(T!['}']) {
            p.error(crate::Error::new(ExpectedBraceEnd, p.current_range()));
        }

        trailing_comment(p);

        m.complete(p, INLINE_TABLE);
    }
}

#[cfg(test)]
mod test {
    use crate::test_parser;

    test_parser! {
        #[test]
        fn empty_inline_table("key = {}") -> Ok(_)
    }

    test_parser! {
        #[test]
        fn inline_table_single_key("key = { key = 1 }") -> Ok(_)
    }

    test_parser! {
        #[test]
        fn inline_table_multi_keys("key = { key = 1, key = 2 }") -> Ok(_)
    }

    test_parser! {
        #[test]
        fn inline_table_multi_keys_with_trailing_comma_v1_1_0(
            "key = { key = 1, key = 2, }"
        ) -> Ok(_)
    }

    test_parser! {
        #[test]
        fn inline_table_multi_line_in_multi_line_value_v1_0_0(r#"
            a = { a = [
            ]}
            b = { a = [
              1,
              2,
       	    ], b = [
              3,
              4,
       	    ]}
            "#
        ) -> Ok(_)
    }

    test_parser! {
        #[test]
        fn inline_table_multi_line_in_v1_1_0(r#"
            key = {
                key1 = 1,
                key2 = 2,
            }
            "#
        ) -> Ok(_)
    }

    test_parser! {
        #[test]
        fn inline_table_multi_line_in_v1_1_0_with_trailing_comment(r#"
            key = { # trailing comment
                key1 = 1, # trailing comment
                key2 = 2,
            } # trailing comment
            "#
        ) -> Ok(_)
    }
}
