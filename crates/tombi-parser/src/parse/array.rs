use tombi_syntax::{SyntaxKind::*, T};

use crate::{
    ErrorKind::*,
    parse::Parse,
    parser::Parser,
    support::{leading_comments, peek_leading_comments, trailing_comment},
    token_set::TS_ARRAY_END,
};

impl Parse for tombi_ast::Array {
    fn parse(p: &mut Parser<'_>) {
        let m = p.start();

        leading_comments(p);

        debug_assert!(p.at(T!['[']));

        p.eat(T!['[']);

        trailing_comment(p);

        loop {
            while p.eat(LINE_BREAK) {}

            Vec::<tombi_ast::DanglingCommentGroup>::parse(p);

            let n = peek_leading_comments(p);
            if p.nth_at_ts(n, TS_ARRAY_END) {
                break;
            }

            tombi_ast::ValueWithCommaGroup::parse(p);
        }

        if !p.eat(T![']']) {
            p.error(crate::Error::new(ExpectedBracketEnd, p.current_range()));
        }

        trailing_comment(p);

        m.complete(p, ARRAY);
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;

    use crate::{ErrorKind::*, test_parser};

    test_parser! {
        #[test]
        fn empty_array("key = []") -> Ok(
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
                        ARRAY: {
                            BRACKET_START: "[",
                            BRACKET_END: "]"
                        }
                    }
                }
            }
        )
    }

    test_parser! {
        #[test]
        fn number_array("key = [1, 2]") -> Ok(
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
                        ARRAY: {
                            BRACKET_START: "[",
                            VALUE_WITH_COMMA_GROUP: {
                                INTEGER_DEC: {
                                    INTEGER_DEC: "1"
                                },
                                COMMA: {
                                    COMMA: ","
                                },
                                WHITESPACE: " ",
                                INTEGER_DEC: {
                                    INTEGER_DEC: "2"
                                }
                            },
                            BRACKET_END: "]"
                        }
                    }
                }
            }
        )
    }

    test_parser! {
        #[test]
        fn number_array_with_trailing_comma("key = [1, 2,]") -> Ok(
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
                        ARRAY: {
                            BRACKET_START: "[",
                            VALUE_WITH_COMMA_GROUP: {
                                INTEGER_DEC: {
                                    INTEGER_DEC: "1"
                                },
                                COMMA: {
                                    COMMA: ","
                                },
                                WHITESPACE: " ",
                                INTEGER_DEC: {
                                    INTEGER_DEC: "2"
                                },
                                COMMA: {
                                    COMMA: ","
                                }
                            },
                            BRACKET_END: "]"
                        }
                    }
                }
            }
        )
    }

    test_parser! {
        #[test]
        fn array_values_with_comma_with_comment_and_line_break(
            r#"
            key = [
              1
              # comma leading
              , # comma trailing
              # value2 leading
              2
            ]
            "#
        ) -> Ok(|root| -> {
            let array = root
                .key_value_groups()
                .find_map(|group| group.into_item_group())
                .and_then(|group| group.key_values().next())
                .and_then(|key_value| key_value.value())
                .and_then(|value| match value {
                    tombi_ast::Value::Array(array) => Some(array),
                    _ => None,
                })
                .unwrap();

            let value_group = array
                .value_with_comma_groups()
                .find_map(|group| group.into_item_group())
                .unwrap();

            let borrowed = value_group
                .values_with_comma()
                .map(|(_, comma)| comma.is_some())
                .collect_vec();
            let owned = value_group
                .clone()
                .into_values_with_comma()
                .map(|(_, comma)| comma.is_some())
                .collect_vec();

            borrowed == vec![true, false] && owned == borrowed
        })
    }

    test_parser! {
        #[test]
        fn array_values_with_comma_without_comma_with_comment_and_line_break(
            r#"
            key = [
              1
              # value2 leading
              2, # comma trailing
              3
            ]
            "#
        ) -> Ok(|root| -> {
            let array = root
                .key_value_groups()
                .find_map(|group| group.into_item_group())
                .and_then(|group| group.key_values().next())
                .and_then(|key_value| key_value.value())
                .and_then(|value| match value {
                    tombi_ast::Value::Array(array) => Some(array),
                    _ => None,
                })
                .unwrap();

            let value_group = array
                .value_with_comma_groups()
                .find_map(|group| group.into_item_group())
                .unwrap();

            let borrowed = value_group
                .values_with_comma()
                .map(|(_, comma)| comma.is_some())
                .collect_vec();
            let owned = value_group
                .clone()
                .into_values_with_comma()
                .map(|(_, comma)| comma.is_some())
                .collect_vec();

            borrowed == vec![false, true, false] && owned == borrowed
        })
    }

    test_parser! {
        #[test]
        fn array_only_key("key = [key]") -> Err([
            SyntaxError(ExpectedValue, 0:7..0:10),
        ])
    }

    test_parser! {
        #[test]
        fn array_only_key_dot("key = [key.]") -> Err([
            SyntaxError(ExpectedValue, 0:7..0:10),
            SyntaxError(ForbiddenKeysLastPeriod, 0:11..0:12),
        ])
    }
}
