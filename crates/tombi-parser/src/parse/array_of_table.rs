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
        ) -> Ok(
            {
                ARRAY_OF_TABLE: {
                    DOUBLE_BRACKET_START: "[[",
                    KEYS: {
                        BARE_KEY: {
                            BARE_KEY: "0x96f"
                        }
                    },
                    DOUBLE_BRACKET_END: "]]",
                    LINE_BREAK: "\n",
                    KEY_VALUE_GROUP: {
                        KEY_VALUE: {
                            KEYS: {
                                BARE_KEY: {
                                    BARE_KEY: "name"
                                }
                            },
                            WHITESPACE: " ",
                            EQUAL: "=",
                            WHITESPACE: " ",
                            BASIC_STRING: {
                                BASIC_STRING: "\"hex-like\""
                            }
                        }
                    }
                }
            }
        )
    }

    test_parser! {
        #[test]
        fn octal_like_array_of_table_key(
            r#"
            [[0o755]]
            mode = "permissions"
            "#
        ) -> Ok(
            {
                ARRAY_OF_TABLE: {
                    DOUBLE_BRACKET_START: "[[",
                    KEYS: {
                        BARE_KEY: {
                            BARE_KEY: "0o755"
                        }
                    },
                    DOUBLE_BRACKET_END: "]]",
                    LINE_BREAK: "\n",
                    KEY_VALUE_GROUP: {
                        KEY_VALUE: {
                            KEYS: {
                                BARE_KEY: {
                                    BARE_KEY: "mode"
                                }
                            },
                            WHITESPACE: " ",
                            EQUAL: "=",
                            WHITESPACE: " ",
                            BASIC_STRING: {
                                BASIC_STRING: "\"permissions\""
                            }
                        }
                    }
                }
            }
        )
    }

    test_parser! {
        #[test]
        fn binary_like_array_of_table_key(
            r#"
            [[0b1010]]
            flags = true
            "#
        ) -> Ok(
            {
                ARRAY_OF_TABLE: {
                    DOUBLE_BRACKET_START: "[[",
                    KEYS: {
                        BARE_KEY: {
                            BARE_KEY: "0b1010"
                        }
                    },
                    DOUBLE_BRACKET_END: "]]",
                    LINE_BREAK: "\n",
                    KEY_VALUE_GROUP: {
                        KEY_VALUE: {
                            KEYS: {
                                BARE_KEY: {
                                    BARE_KEY: "flags"
                                }
                            },
                            WHITESPACE: " ",
                            EQUAL: "=",
                            WHITESPACE: " ",
                            BOOLEAN: {
                                BOOLEAN: "true"
                            }
                        }
                    }
                }
            }
        )
    }

    test_parser! {
        #[test]
        fn parses_array_of_table_dangling_comment(
            r#"
            [[header]]
            # dangling comment
            "#
        ) -> Ok(
            {
                ARRAY_OF_TABLE: {
                    DOUBLE_BRACKET_START: "[[",
                    KEYS: {
                        BARE_KEY: {
                            BARE_KEY: "header"
                        }
                    },
                    DOUBLE_BRACKET_END: "]]",
                    LINE_BREAK: "\n",
                    DANGLING_COMMENT_GROUP: {
                        COMMENT: "# dangling comment"
                    }
                }
            }
        )
    }

    test_parser! {
        #[test]
        fn parses_array_of_table_new_line_dangling_comment(
            r#"
            [[header]]

            # dangling comment
            "#
        ) -> Ok(
            {
                ARRAY_OF_TABLE: {
                    DOUBLE_BRACKET_START: "[[",
                    KEYS: {
                        BARE_KEY: {
                            BARE_KEY: "header"
                        }
                    },
                    DOUBLE_BRACKET_END: "]]",
                    LINE_BREAK: "\n",
                    LINE_BREAK: "\n",
                    DANGLING_COMMENT_GROUP: {
                        COMMENT: "# dangling comment"
                    }
                }
            }
        )
    }

    test_parser! {
        #[test]
        fn parses_array_of_table_dangling_comment_group(
            r#"
            [[header]]
            # dangling comment group 1
            # dangling comment group 1
            "#
        ) -> Ok(
            {
                ARRAY_OF_TABLE: {
                    DOUBLE_BRACKET_START: "[[",
                    KEYS: {
                        BARE_KEY: {
                            BARE_KEY: "header"
                        }
                    },
                    DOUBLE_BRACKET_END: "]]",
                    LINE_BREAK: "\n",
                    DANGLING_COMMENT_GROUP: {
                        COMMENT: "# dangling comment group 1",
                        LINE_BREAK: "\n",
                        COMMENT: "# dangling comment group 1"
                    }
                }
            }
        )
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
        ) -> Ok(
            {
                ARRAY_OF_TABLE: {
                    DOUBLE_BRACKET_START: "[[",
                    KEYS: {
                        BARE_KEY: {
                            BARE_KEY: "header"
                        }
                    },
                    DOUBLE_BRACKET_END: "]]",
                    LINE_BREAK: "\n",
                    DANGLING_COMMENT_GROUP: {
                        COMMENT: "# dangling comment group 1",
                        LINE_BREAK: "\n",
                        COMMENT: "# dangling comment group 1"
                    },
                    LINE_BREAK: "\n",
                    LINE_BREAK: "\n",
                    DANGLING_COMMENT_GROUP: {
                        COMMENT: "# dangling comment group 2",
                        LINE_BREAK: "\n",
                        COMMENT: "# dangling comment group 2"
                    },
                    LINE_BREAK: "\n",
                    LINE_BREAK: "\n",
                    LINE_BREAK: "\n",
                    DANGLING_COMMENT_GROUP: {
                        COMMENT: "# dangling comment group 3",
                        LINE_BREAK: "\n",
                        COMMENT: "# dangling comment group 3"
                    }
                }
            }
        )
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
        ) -> Ok(
            {
                ARRAY_OF_TABLE: {
                    DOUBLE_BRACKET_START: "[[",
                    KEYS: {
                        BARE_KEY: {
                            BARE_KEY: "header"
                        }
                    },
                    DOUBLE_BRACKET_END: "]]",
                    LINE_BREAK: "\n",
                    KEY_VALUE_GROUP: {
                        KEY_VALUE: {
                            KEYS: {
                                BARE_KEY: {
                                    BARE_KEY: "key1"
                                }
                            },
                            WHITESPACE: " ",
                            EQUAL: "=",
                            WHITESPACE: " ",
                            BASIC_STRING: {
                                BASIC_STRING: "\"value1\""
                            }
                        },
                        KEY_VALUE: {
                            LINE_BREAK: "\n",
                            KEYS: {
                                BARE_KEY: {
                                    BARE_KEY: "key2"
                                }
                            },
                            WHITESPACE: " ",
                            EQUAL: "=",
                            WHITESPACE: " ",
                            BASIC_STRING: {
                                BASIC_STRING: "\"value2\""
                            }
                        }
                    },
                    LINE_BREAK: "\n",
                    DANGLING_COMMENT_GROUP: {
                        COMMENT: "# dangling comment group 1",
                        LINE_BREAK: "\n",
                        COMMENT: "# dangling comment group 1"
                    },
                    LINE_BREAK: "\n",
                    LINE_BREAK: "\n",
                    DANGLING_COMMENT_GROUP: {
                        COMMENT: "# dangling comment group 2",
                        LINE_BREAK: "\n",
                        COMMENT: "# dangling comment group 2"
                    },
                    LINE_BREAK: "\n",
                    LINE_BREAK: "\n",
                    KEY_VALUE_GROUP: {
                        KEY_VALUE: {
                            KEYS: {
                                BARE_KEY: {
                                    BARE_KEY: "key3"
                                }
                            },
                            WHITESPACE: " ",
                            EQUAL: "=",
                            WHITESPACE: " ",
                            BASIC_STRING: {
                                BASIC_STRING: "\"value3\""
                            }
                        },
                        KEY_VALUE: {
                            LINE_BREAK: "\n",
                            KEYS: {
                                BARE_KEY: {
                                    BARE_KEY: "key4"
                                }
                            },
                            WHITESPACE: " ",
                            EQUAL: "=",
                            WHITESPACE: " ",
                            BASIC_STRING: {
                                BASIC_STRING: "\"value4\""
                            }
                        }
                    },
                    LINE_BREAK: "\n",
                    LINE_BREAK: "\n",
                    KEY_VALUE_GROUP: {
                        KEY_VALUE: {
                            COMMENT: "# leading comment 1",
                            LINE_BREAK: "\n",
                            COMMENT: "# leading comment 1",
                            LINE_BREAK: "\n",
                            KEYS: {
                                BARE_KEY: {
                                    BARE_KEY: "key5"
                                }
                            },
                            WHITESPACE: " ",
                            EQUAL: "=",
                            WHITESPACE: " ",
                            BASIC_STRING: {
                                BASIC_STRING: "\"value5\""
                            }
                        },
                        KEY_VALUE: {
                            LINE_BREAK: "\n",
                            COMMENT: "# leading comment 2",
                            LINE_BREAK: "\n",
                            KEYS: {
                                BARE_KEY: {
                                    BARE_KEY: "key6"
                                }
                            },
                            WHITESPACE: " ",
                            EQUAL: "=",
                            WHITESPACE: " ",
                            BASIC_STRING: {
                                BASIC_STRING: "\"value6\""
                            }
                        }
                    },
                    LINE_BREAK: "\n",
                    LINE_BREAK: "\n",
                    DANGLING_COMMENT_GROUP: {
                        COMMENT: "# dangling comment group 3",
                        LINE_BREAK: "\n",
                        COMMENT: "# dangling comment group 3"
                    }
                }
            }
        )
    }
}
