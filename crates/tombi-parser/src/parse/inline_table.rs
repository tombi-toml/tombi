use tombi_syntax::{SyntaxKind::*, T};

use crate::{
    ErrorKind::*,
    parse::Parse,
    parser::Parser,
    support::{leading_comments, peek_leading_comments, trailing_comment},
    token_set::TS_INLINE_TABLE_END,
};

impl Parse for tombi_ast::InlineTable {
    fn parse(p: &mut Parser<'_>) {
        let m = p.start();

        leading_comments(p);

        debug_assert!(p.at(T!['{']));

        p.eat(T!['{']);

        trailing_comment(p);

        loop {
            while p.eat(LINE_BREAK) {}

            Vec::<tombi_ast::DanglingCommentGroup>::parse(p);

            let n = peek_leading_comments(p);
            if p.nth_at_ts(n, TS_INLINE_TABLE_END) {
                break;
            }

            tombi_ast::KeyValueWithCommaGroup::parse(p);
        }

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
        fn inline_table_multi_line_in_multi_line_value_v1_0_0(
            r#"
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
        fn inline_table_multi_line_in_v1_1_0(
            r#"
            key = {
                key1 = 1,
                key2 = 2,
            }
            "#
        ) -> Ok(_)
    }

    test_parser! {
        #[test]
        fn inline_table_multi_line_in_v1_1_0_with_trailing_comment(
            r#"
            key = { # trailing comment
                key1 = 1, # trailing comment
                key2 = 2,
            } # trailing comment
            "#
        ) -> Ok(_)
    }

    test_parser! {
        #[test]
        fn inline_table_dangling_comment(
            r#"
            key = {
                # dangling comment
            }
            "#
        ) -> Ok(
            {
                KEY_VALUE_GROUP: {
                    KEY_VALUE: {
                        KEYS: {
                            BARE_KEY: {
                                BARE_KEY: "key"
                            }
                        },
                        WHITESPACE: " ",
                        EQUAL: "=",
                        WHITESPACE: " ",
                        INLINE_TABLE: {
                            BRACE_START: "{",
                            LINE_BREAK: "\n",
                            WHITESPACE: "    ",
                            DANGLING_COMMENT_GROUP: {
                                COMMENT: "# dangling comment"
                            },
                            LINE_BREAK: "\n",
                            BRACE_END: "}"
                        }
                    }
                }
            }
        )
    }

    test_parser! {
        #[test]
        fn inline_table_new_line_dangling_comment(
            r#"
            key = {

                # dangling comment
            }
            "#
        ) -> Ok(_)
    }

    test_parser! {
        #[test]
        fn inline_table_dangling_comment_groups(
            r#"
            key = {
                # dangling comment group 1
                # dangling comment group 1

                # dangling comment group 2
                # dangling comment group 2


                # dangling comment group 3
                # dangling comment group 3
            }
            "#
        ) -> Ok(_)
    }

    test_parser! {
        #[test]
        fn inline_table_key_value_with_comma_group_and_dangling_comment_groups(
            r#"
            key = {
                key1 = "value1",
                key2 = "value2",
                # dangling comment group 1
                # dangling comment group 1

                # dangling comment group 2
                # dangling comment group 2

                key3 = "value3",
                key4 = "value4",

                # leading comment 1
                # leading comment 1
                key5 = "value5",
                # leading comment 2
                key6 = "value6",

                # dangling comment group 3
                # dangling comment group 3
            }
            "#
        ) -> Ok(_)
    }
}
