use tombi_syntax::SyntaxKind::*;
use tombi_syntax::T;

use crate::{
    ErrorKind::*,
    parse::{Parse, TS_LINE_END, invalid_line},
    parser::Parser,
    support::{leading_comments, peek_leading_comments, trailing_comment},
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

        loop {
            while p.eat(LINE_BREAK) {}

            Vec::<tombi_ast::DanglingCommentGroup>::parse(p);

            let n = peek_leading_comments(p);
            if p.nth_at_ts(n, TS_NEXT_SECTION) {
                break;
            }

            tombi_ast::KeyValueGroup::parse(p);

            if !p.at_ts(TS_LINE_END) {
                invalid_line(p, ExpectedLineBreak);
            }
        }

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

    test_parser! {
        #[test]
        fn keeps_key_value_leading_comments_as_non_dangling(
            r#"
            [table]
            # leading comment
            key = 1
            "#
        ) -> Ok(|root| -> {
            let table = root.items().find_map(|item| match item {
                tombi_ast::RootItem::Table(table) => Some(table),
                _ => None,
            });

            table
                .map(|table| table.dangling_comment_groups().count() == 0 && table.key_value_groups().count() == 1)
                .unwrap_or(false)
        })
    }

    test_parser! {
        #[test]
        fn parses_table_dangling_comment(
            r#"
            [header]
            # dangling comment
            "#
        ) -> Ok(_)
    }

    test_parser! {
        #[test]
        fn parses_table_new_line_dangling_comment(
            r#"
            [header]

            # dangling comment
            "#
        ) -> Ok(_)
    }

    test_parser! {
        #[test]
        fn parses_table_dangling_comment_group(
            r#"
            [header]
            # dangling comment group 1
            # dangling comment group 1
            "#
        ) -> Ok(_)
    }

    test_parser! {
        #[test]
        fn parses_table_dangling_comment_groups(
            r#"
            [header]
            # dangling comment group 1
            # dangling comment group 1

            # dangling comment group 2
            # dangling comment group 2


            # dangling comment group 3
            # dangling comment group 3
            "#
        ) -> Ok(_)
    }

    test_parser! {
        #[test]
        fn parses_table_key_value_group_and_dangling_comment_groups(
            r#"
            [header]
            key1 = "value1"
            key2 = "value2"
            # dangling comment group 1
            # dangling comment group 1

            # dangling comment group 2
            # dangling comment group 2

            key3 = "value3"
            key4 = "value4"

            # leading comment 1
            # leading comment 1
            key5 = "value5"
            # leading comment 2
            key6 = "value6"

            # dangling comment group 3
            # dangling comment group 3
            "#
        ) -> Ok(_)
    }
}
