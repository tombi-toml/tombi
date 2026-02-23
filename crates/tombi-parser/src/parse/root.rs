use tombi_syntax::{SyntaxKind::*, T};

use super::{Parse, TS_LINE_END, invalid_line};
use crate::{
    ErrorKind::*,
    parser::Parser,
    support::{leading_comments, peek_leading_comments, trailing_comment},
    token_set::TS_NEXT_SECTION,
};

impl Parse for tombi_ast::Root {
    fn parse(p: &mut Parser<'_>) {
        let m = p.start();

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

        loop {
            while p.eat(LINE_BREAK) {}

            let n = peek_leading_comments(p);
            if p.nth_at(n, EOF) {
                break;
            } else if p.nth_at(n, T!("[[")) {
                tombi_ast::ArrayOfTable::parse(p);
            } else if p.nth_at(n, T!['[']) {
                tombi_ast::Table::parse(p);
            } else {
                unknwon_line(p);
            }
        }

        m.complete(p, ROOT);
    }
}

fn unknwon_line(p: &mut Parser<'_>) {
    let m = p.start();

    leading_comments(p);

    while !p.at_ts(TS_LINE_END) {
        p.bump_any();
    }
    p.error(crate::Error::new(UnknownLine, p.current_range()));

    trailing_comment(p);

    m.complete(p, ERROR);
}

#[cfg(test)]
mod test {
    use crate::test_parser;

    test_parser! {
        #[test]
        fn parses_root_begin_dangling_comments(
            r#"
            # begin dangling_comment1
            # begin dangling_comment2

            # table leading comment1
            # table leading comment2
            [table]
            "#
        ) -> Ok(
            {
                DANGLING_COMMENT_GROUP: {
                    COMMENT: "# begin dangling_comment1",
                    LINE_BREAK: "\n",
                    COMMENT: "# begin dangling_comment2"
                },
                LINE_BREAK: "\n",
                LINE_BREAK: "\n",
                TABLE: {
                    COMMENT: "# table leading comment1",
                    LINE_BREAK: "\n",
                    COMMENT: "# table leading comment2",
                    LINE_BREAK: "\n",
                    BRACKET_START: "[",
                    KEYS: {
                        BARE_KEY: {
                            BARE_KEY: "table"
                        }
                    },
                    BRACKET_END: "]"
                }
            }
        )
    }

    test_parser! {
        #[test]
        fn parses_root_dangling_comment(
            r#"
            # dangling comment
            "#
        ) -> Ok(
            {
                DANGLING_COMMENT_GROUP: {
                    COMMENT: "# dangling comment"
                }
            }
        )
    }

    test_parser! {
        #[test]
        fn parses_root_dangling_comment_groups(
            r#"
            # dangling comment group 1
            # dangling comment group 1

            # dangling comment group 2
            # dangling comment group 2


            # dangling comment group 3
            # dangling comment group 3
            "#
        ) -> Ok(
            {
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
        )
    }

    test_parser! {
        #[test]
        fn parses_root_key_value_group_and_dangling_comment_groups(
            r#"
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
        )
    }

    test_parser! {
        #[test]
        fn parses_root_key_values_then_tables(
            r#"
            key1 = "value1"
            key2 = "value2"

            [table1]
            key3 = "value3"

            [[array_of_table1]]
            key4 = "value4"
            "#
        ) -> Ok(
            {
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
                LINE_BREAK: "\n",
                TABLE: {
                    BRACKET_START: "[",
                    KEYS: {
                        BARE_KEY: {
                            BARE_KEY: "table1"
                        }
                    },
                    BRACKET_END: "]",
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
                        }
                    },
                    LINE_BREAK: "\n",
                    LINE_BREAK: "\n"
                },
                ARRAY_OF_TABLE: {
                    DOUBLE_BRACKET_START: "[[",
                    KEYS: {
                        BARE_KEY: {
                            BARE_KEY: "array_of_table1"
                        }
                    },
                    DOUBLE_BRACKET_END: "]]",
                    LINE_BREAK: "\n",
                    KEY_VALUE_GROUP: {
                        KEY_VALUE: {
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
                    }
                }
            }
        )
    }

    test_parser! {
        #[test]
        fn parses_root_dangling_comments_then_tables(
            r#"
            # dangling comment group 1
            # dangling comment group 1

            # dangling comment group 2
            # dangling comment group 2

            # table leading comment
            [table1]
            key1 = "value1"
            "#
        ) -> Ok(
            {
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
                TABLE: {
                    COMMENT: "# table leading comment",
                    LINE_BREAK: "\n",
                    BRACKET_START: "[",
                    KEYS: {
                        BARE_KEY: {
                            BARE_KEY: "table1"
                        }
                    },
                    BRACKET_END: "]",
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
                        }
                    }
                }
            }
        )
    }
}
