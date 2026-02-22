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
        fn array_only_key("key = [key]") -> Err([
            SyntaxError(ExpectedValue, 0:7..0:10),
        ])
    }

    test_parser! {
        #[test]
        fn array_only_key_dot("key = [key.]") -> Err([
            SyntaxError(ExpectedValue, 0:7..0:10),
            SyntaxError(ExpectedComma, 0:10..0:11),
        ])
    }
}
