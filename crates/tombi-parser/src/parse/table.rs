use tombi_syntax::{SyntaxKind::*, T};

use crate::{
    ErrorKind::*,
    parse::{
        Parse, TS_LINE_END, begin_dangling_comments, end_dangling_comments, invalid_line,
        leading_comments, peek_leading_comments, trailing_comment,
    },
    parser::Parser,
    token_set::TS_NEXT_SECTION,
};

impl Parse for tombi_ast::Table {
    fn parse(p: &mut Parser<'_>) {
        let m = p.start();

        leading_comments(p);

        debug_assert!(p.at(T!['[']));

        p.eat(T!['[']);

        tombi_ast::Keys::parse(p);

        if !p.eat(T![']']) {
            invalid_line(p, ExpectedBracketEnd);
        }

        trailing_comment(p);

        if !p.at_ts(TS_LINE_END) {
            invalid_line(p, ExpectedLineBreak);
        }
        p.eat(LINE_BREAK);

        begin_dangling_comments(p);

        loop {
            while p.eat(LINE_BREAK) {}
            let n = peek_leading_comments(p);

            if p.nth_at_ts(n, TS_NEXT_SECTION) {
                break;
            }

            tombi_ast::KeyValue::parse(p);

            if !p.at_ts(TS_LINE_END) {
                invalid_line(p, ExpectedLineBreak);
            }
        }

        end_dangling_comments(p, false);

        // NOTE: For easier calculation of the table range
        //       from the cursor position in the editor,
        //       consume whitespace until the next section.
        while p.eat(LINE_BREAK) {}

        m.complete(p, TABLE);
    }
}

#[cfg(test)]
mod test {
    use crate::{ErrorKind::*, test_parser};

    test_parser! {
        #[test]
        fn without_header_keys(
            r#"
                []
                key1 = 1
                key2 = 2
                "#
        ) -> Err([
            SyntaxError(ExpectedKey, 0:1..0:2),
        ])
    }

    test_parser! {
        #[test]
        fn without_last_dot_key(
            r#"
            [aaa.]
            key1 = 1
            key2 = 2
            "#
        ) -> Err([
            SyntaxError(ForbiddenKeysLastPeriod, 0:5..0:6),
        ])
    }

    test_parser! {
        #[test]
        fn without_last_bracket(
            r#"
            [aaa.bbb
            key1 = 1
            key2 = 2
            "#
        ) -> Err([
            SyntaxError(ExpectedBracketEnd, 0:8..1:0),
        ])
    }

    test_parser! {
        #[test]
        fn without_value(
            r#"
            [aaa.bbb]
            key1 = 1
            key2 = 2

            [aaa.ccc]
            key1 =
            key2 = 2

            [aaa.ddd]
            key1 = 1
            key2 = 2
            "#
        ) -> Err([
            SyntaxError(ExpectedValue, 5:6..6:0),
        ])
    }

    test_parser! {
        #[test]
        fn invalid_key_value_trailing_comment(
            r#"
            [aaa.bbb]
            key1 = 1 INVALID COMMENT
            key2 = 2
            "#
        ) -> Err([
            SyntaxError(ExpectedLineBreak, 1:9..1:16),
        ])
    }

    test_parser! {
        #[test]
        fn hex_like_table_key(
            r#"
            [0x96f]
            submodule = "extensions/0x96f"
            version = "1.3.5"
            "#
        ) -> Ok(_)
    }

    test_parser! {
        #[test]
        fn octal_like_table_key(
            r#"
            [0o755]
            value = "octal key"
            "#
        ) -> Ok(_)
    }

    test_parser! {
        #[test]
        fn binary_like_table_key(
            r#"
            [0b1010]
            value = "binary key"
            "#
        ) -> Ok(_)
    }
}
