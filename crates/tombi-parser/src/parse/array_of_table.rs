use tombi_syntax::{SyntaxKind::*, T};

use super::Parse;
use crate::{
    ErrorKind::*,
    parse::{TS_LINE_END, invalid_line},
    parser::Parser,
    support::{leading_comments, peek_leading_comments, trailing_comment},
    token_set::TS_NEXT_SECTION,
};

impl Parse for tombi_ast::ArrayOfTable {
    fn parse(p: &mut Parser<'_>) {
        let m = p.start();

        leading_comments(p);

        debug_assert!(p.at(T!("[[")));

        p.eat(T!("[["));

        tombi_ast::Keys::parse(p);

        if !p.eat(T!("]]")) {
            invalid_line(p, ExpectedDoubleBracketEnd);
        }

        trailing_comment(p);

        if !p.at_ts(TS_LINE_END) {
            invalid_line(p, ExpectedLineBreak);
        }
        p.eat(LINE_BREAK);

        loop {
            while p.eat(LINE_BREAK) {}

            Vec::<tombi_ast::DanglingCommentGroup>::parse(p);

            let n = peek_leading_comments(p);
            if p.nth_at_ts(n, TS_NEXT_SECTION) {
                break;
            }

            tombi_ast::KeyValue::parse(p);

            if !p.at_ts(TS_LINE_END) {
                invalid_line(p, ExpectedLineBreak);
            }
        }

        m.complete(p, ARRAY_OF_TABLE);
    }
}

#[cfg(test)]
mod test {
    use crate::{ErrorKind::*, test_parser};

    test_parser! {
        #[test]
        fn invalid_array_of_table1(
            r#"
            [[]]
            key1 = 1
            key2 = 2
            "#
        ) -> Err([SyntaxError(ExpectedKey, 0:2..0:3)])
    }

    test_parser! {
        #[test]
        fn invalid_array_of_table2(
            r#"
            [[aaa.]]
            key1 = 1
            key2 = 2
            "#
        ) -> Err([SyntaxError(ForbiddenKeysLastPeriod, 0:6..0:7)])
    }

    test_parser! {
        #[test]
        fn invalid_array_of_table3(
            r#"
            [[aaa.bbb
            key1 = 1
            key2 = 2
            "#
        ) -> Err([SyntaxError(ExpectedDoubleBracketEnd, 0:9..1:0)])
    }

    test_parser! {
        #[test]
        fn invalid_array_of_table4(
            r#"
            [[aaa.bbb]
            key1 = 1
            key2 = 2
            "#
        ) -> Err([SyntaxError(ExpectedDoubleBracketEnd, 0:9..0:10)])
    }

    test_parser! {
        #[test]
        fn invalid_array_of_table5(
            r#"
            [[aaa.bbb]]
            key1 = 1 INVALID COMMENT
            key2 = 2
            "#
        ) -> Err([SyntaxError(ExpectedLineBreak, 1:9..1:16)])
    }

    test_parser! {
        #[test]
        fn hex_like_array_of_table_key(
            r#"
            [[0x96f]]
            name = "hex-like"
            "#
        ) -> Ok(_)
    }

    test_parser! {
        #[test]
        fn octal_like_array_of_table_key(
            r#"
            [[0o755]]
            mode = "permissions"
            "#
        ) -> Ok(_)
    }

    test_parser! {
        #[test]
        fn binary_like_array_of_table_key(
            r#"
            [[0b1010]]
            flags = true
            "#
        ) -> Ok(_)
    }

    test_parser! {
        #[test]
        fn parses_array_of_table_dangling_comment(
            r#"
            [[header]]
            # dangling comment
            "#
        ) -> Ok(_)
    }

    test_parser! {
        #[test]
        fn parses_array_of_table_new_line_dangling_comment(
            r#"
            [[header]]

            # dangling comment
            "#
        ) -> Ok(_)
    }

    test_parser! {
        #[test]
        fn parses_array_of_table_dangling_comment_group(
            r#"
            [[header]]
            # dangling comment group 1
            # dangling comment group 1
            "#
        ) -> Ok(_)
    }

    test_parser! {
        #[test]
        fn parses_array_of_table_dangling_comment_groups(
            r#"
            [[header]]
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
        fn parses_array_of_table_key_value_group_and_dangling_comment_groups(
            r#"
            [[header]]
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
