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
    use itertools::Itertools;

    use crate::{ErrorKind::*, test_parser};

    test_parser! {
        #[test]
        fn empty_inline_table("key = {}") -> Ok(
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
                            BRACE_END: "}"
                        }
                    }
                }
            }
        )
    }

    test_parser! {
        #[test]
        fn inline_table_single_key("key = { key = 1 }") -> Ok(
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
                            WHITESPACE: " ",
                            KEY_VALUE_WITH_COMMA_GROUP: {
                                KEY_VALUE: {
                                    KEYS: {
                                        BARE_KEY: {
                                            BARE_KEY: "key"
                                        }
                                    },
                                    WHITESPACE: " ",
                                    EQUAL: "=",
                                    WHITESPACE: " ",
                                    INTEGER_DEC: {
                                        INTEGER_DEC: "1"
                                    }
                                }
                            },
                            WHITESPACE: " ",
                            BRACE_END: "}"
                        }
                    }
                }
            }
        )
    }

    test_parser! {
        #[test]
        fn inline_table_multi_keys("key = { key = 1, key = 2 }") -> Ok(
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
                            WHITESPACE: " ",
                            KEY_VALUE_WITH_COMMA_GROUP: {
                                KEY_VALUE: {
                                    KEYS: {
                                        BARE_KEY: {
                                            BARE_KEY: "key"
                                        }
                                    },
                                    WHITESPACE: " ",
                                    EQUAL: "=",
                                    WHITESPACE: " ",
                                    INTEGER_DEC: {
                                        INTEGER_DEC: "1"
                                    }
                                },
                                COMMA: {
                                    COMMA: ","
                                },
                                WHITESPACE: " ",
                                KEY_VALUE: {
                                    KEYS: {
                                        BARE_KEY: {
                                            BARE_KEY: "key"
                                        }
                                    },
                                    WHITESPACE: " ",
                                    EQUAL: "=",
                                    WHITESPACE: " ",
                                    INTEGER_DEC: {
                                        INTEGER_DEC: "2"
                                    }
                                }
                            },
                            WHITESPACE: " ",
                            BRACE_END: "}"
                        }
                    }
                }
            }
        )
    }

    test_parser! {
        #[test]
        fn inline_table_multi_keys_with_trailing_comma(
            "key = { key = 1, key = 2, }"
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
                            WHITESPACE: " ",
                            KEY_VALUE_WITH_COMMA_GROUP: {
                                KEY_VALUE: {
                                    KEYS: {
                                        BARE_KEY: {
                                            BARE_KEY: "key"
                                        }
                                    },
                                    WHITESPACE: " ",
                                    EQUAL: "=",
                                    WHITESPACE: " ",
                                    INTEGER_DEC: {
                                        INTEGER_DEC: "1"
                                    }
                                },
                                COMMA: {
                                    COMMA: ","
                                },
                                WHITESPACE: " ",
                                KEY_VALUE: {
                                    KEYS: {
                                        BARE_KEY: {
                                            BARE_KEY: "key"
                                        }
                                    },
                                    WHITESPACE: " ",
                                    EQUAL: "=",
                                    WHITESPACE: " ",
                                    INTEGER_DEC: {
                                        INTEGER_DEC: "2"
                                    }
                                },
                                COMMA: {
                                    COMMA: ","
                                }
                            },
                            WHITESPACE: " ",
                            BRACE_END: "}"
                        }
                    }
                }
            }
        )
    }

    test_parser! {
        #[test]
        fn inline_table_key_values_with_comma_with_comment_and_line_break(
            r#"
            key = {
              a = 1
              # comma leading
              , # comma trailing
              # key b leading
              b = 2
            }
            "#
        ) -> Ok(|root| -> {
            let table = root
                .key_value_groups()
                .find_map(|group| group.into_item_group())
                .and_then(|group| group.key_values().next())
                .and_then(|key_value| key_value.value())
                .and_then(|value| match value {
                    tombi_ast::Value::InlineTable(table) => Some(table),
                    _ => None,
                })
                .unwrap();

            let key_value_group = table
                .key_value_with_comma_groups()
                .find_map(|group| group.into_item_group())
                .unwrap();

            let borrowed = key_value_group
                .key_values_with_comma()
                .map(|(_, comma)| comma.is_some())
                .collect_vec();
            let owned = key_value_group
                .clone()
                .into_key_values_with_comma()
                .map(|(_, comma)| comma.is_some())
                .collect_vec();

            borrowed == vec![true, false] && owned == borrowed
        })
    }

    test_parser! {
        #[test]
        fn inline_table_key_values_with_comma_without_comma_with_comment_and_line_break(
            r#"
            key = {
              a = 1
              # key b leading
              b = 2, # comma trailing
              c = 3
            }
            "#
        ) -> Ok(|root| -> {
            let table = root
                .key_value_groups()
                .find_map(|group| group.into_item_group())
                .and_then(|group| group.key_values().next())
                .and_then(|key_value| key_value.value())
                .and_then(|value| match value {
                    tombi_ast::Value::InlineTable(table) => Some(table),
                    _ => None,
                })
                .unwrap();

            let key_value_group = table
                .key_value_with_comma_groups()
                .find_map(|group| group.into_item_group())
                .unwrap();

            let borrowed = key_value_group
                .key_values_with_comma()
                .map(|(_, comma)| comma.is_some())
                .collect_vec();
            let owned = key_value_group
                .clone()
                .into_key_values_with_comma()
                .map(|(_, comma)| comma.is_some())
                .collect_vec();

            borrowed == vec![false, true, false] && owned == borrowed
        })
    }

    test_parser! {
        #[test]
        fn inline_table_only_key_dot("key = { key = 1. }") -> Err([
            SyntaxError(ExpectedValue, 0:14..0:15),
            SyntaxError(ForbiddenKeysLastPeriod, 0:17..0:18),
        ])
    }

    test_parser! {
        #[test]
        fn inline_table_multi_line_in_multi_line_value(
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
        ) -> Ok(
            {
                KEY_VALUE_GROUP: {
                    KEY_VALUE: {
                        KEYS: {
                            BARE_KEY: {
                                BARE_KEY: "a"
                            }
                        },
                        WHITESPACE: " ",
                        EQUAL: "=",
                        WHITESPACE: " ",
                        INLINE_TABLE: {
                            BRACE_START: "{",
                            WHITESPACE: " ",
                            KEY_VALUE_WITH_COMMA_GROUP: {
                                KEY_VALUE: {
                                    KEYS: {
                                        BARE_KEY: {
                                            BARE_KEY: "a"
                                        }
                                    },
                                    WHITESPACE: " ",
                                    EQUAL: "=",
                                    WHITESPACE: " ",
                                    ARRAY: {
                                        BRACKET_START: "[",
                                        LINE_BREAK: "\n",
                                        WHITESPACE: "     ",
                                        BRACKET_END: "]"
                                    }
                                }
                            },
                            BRACE_END: "}"
                        }
                    },
                    KEY_VALUE: {
                        LINE_BREAK: "\n",
                        WHITESPACE: "     ",
                        KEYS: {
                            BARE_KEY: {
                                BARE_KEY: "b"
                            }
                        },
                        WHITESPACE: " ",
                        EQUAL: "=",
                        WHITESPACE: " ",
                        INLINE_TABLE: {
                            BRACE_START: "{",
                            WHITESPACE: " ",
                            KEY_VALUE_WITH_COMMA_GROUP: {
                                KEY_VALUE: {
                                    KEYS: {
                                        BARE_KEY: {
                                            BARE_KEY: "a"
                                        }
                                    },
                                    WHITESPACE: " ",
                                    EQUAL: "=",
                                    WHITESPACE: " ",
                                    ARRAY: {
                                        BRACKET_START: "[",
                                        LINE_BREAK: "\n",
                                        WHITESPACE: "       ",
                                        VALUE_WITH_COMMA_GROUP: {
                                            INTEGER_DEC: {
                                                INTEGER_DEC: "1"
                                            },
                                            COMMA: {
                                                COMMA: ","
                                            },
                                            INTEGER_DEC: {
                                                LINE_BREAK: "\n",
                                                WHITESPACE: "       ",
                                                INTEGER_DEC: "2"
                                            },
                                            COMMA: {
                                                COMMA: ","
                                            }
                                        },
                                        LINE_BREAK: "\n",
                                        WHITESPACE: "\t    ",
                                        BRACKET_END: "]"
                                    }
                                },
                                COMMA: {
                                    COMMA: ","
                                },
                                WHITESPACE: " ",
                                KEY_VALUE: {
                                    KEYS: {
                                        BARE_KEY: {
                                            BARE_KEY: "b"
                                        }
                                    },
                                    WHITESPACE: " ",
                                    EQUAL: "=",
                                    WHITESPACE: " ",
                                    ARRAY: {
                                        BRACKET_START: "[",
                                        LINE_BREAK: "\n",
                                        WHITESPACE: "       ",
                                        VALUE_WITH_COMMA_GROUP: {
                                            INTEGER_DEC: {
                                                INTEGER_DEC: "3"
                                            },
                                            COMMA: {
                                                COMMA: ","
                                            },
                                            INTEGER_DEC: {
                                                LINE_BREAK: "\n",
                                                WHITESPACE: "       ",
                                                INTEGER_DEC: "4"
                                            },
                                            COMMA: {
                                                COMMA: ","
                                            }
                                        },
                                        LINE_BREAK: "\n",
                                        WHITESPACE: "\t    ",
                                        BRACKET_END: "]"
                                    }
                                }
                            },
                            BRACE_END: "}"
                        }
                    }
                }
            }
        )
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
                            KEY_VALUE_WITH_COMMA_GROUP: {
                                KEY_VALUE: {
                                    KEYS: {
                                        BARE_KEY: {
                                            BARE_KEY: "key1"
                                        }
                                    },
                                    WHITESPACE: " ",
                                    EQUAL: "=",
                                    WHITESPACE: " ",
                                    INTEGER_DEC: {
                                        INTEGER_DEC: "1"
                                    }
                                },
                                COMMA: {
                                    COMMA: ","
                                },
                                KEY_VALUE: {
                                    LINE_BREAK: "\n",
                                    WHITESPACE: "    ",
                                    KEYS: {
                                        BARE_KEY: {
                                            BARE_KEY: "key2"
                                        }
                                    },
                                    WHITESPACE: " ",
                                    EQUAL: "=",
                                    WHITESPACE: " ",
                                    INTEGER_DEC: {
                                        INTEGER_DEC: "2"
                                    }
                                },
                                COMMA: {
                                    COMMA: ","
                                }
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
        fn inline_table_multi_line_in_v1_1_0_with_trailing_comment(
            r#"
            key = { # trailing comment
                key1 = 1, # trailing comment
                key2 = 2,
            } # trailing comment
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
                            WHITESPACE: " ",
                            COMMENT: "# trailing comment",
                            LINE_BREAK: "\n",
                            WHITESPACE: "    ",
                            KEY_VALUE_WITH_COMMA_GROUP: {
                                KEY_VALUE: {
                                    KEYS: {
                                        BARE_KEY: {
                                            BARE_KEY: "key1"
                                        }
                                    },
                                    WHITESPACE: " ",
                                    EQUAL: "=",
                                    WHITESPACE: " ",
                                    INTEGER_DEC: {
                                        INTEGER_DEC: "1"
                                    }
                                },
                                COMMA: {
                                    COMMA: ",",
                                    WHITESPACE: " ",
                                    COMMENT: "# trailing comment"
                                },
                                KEY_VALUE: {
                                    LINE_BREAK: "\n",
                                    WHITESPACE: "    ",
                                    KEYS: {
                                        BARE_KEY: {
                                            BARE_KEY: "key2"
                                        }
                                    },
                                    WHITESPACE: " ",
                                    EQUAL: "=",
                                    WHITESPACE: " ",
                                    INTEGER_DEC: {
                                        INTEGER_DEC: "2"
                                    }
                                },
                                COMMA: {
                                    COMMA: ","
                                }
                            },
                            LINE_BREAK: "\n",
                            BRACE_END: "}",
                            WHITESPACE: " ",
                            COMMENT: "# trailing comment"
                        }
                    }
                }
            }
        )
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
                                COMMENT: "# dangling comment group 1",
                                LINE_BREAK: "\n",
                                WHITESPACE: "    ",
                                COMMENT: "# dangling comment group 1"
                            },
                            LINE_BREAK: "\n",
                            LINE_BREAK: "\n",
                            WHITESPACE: "    ",
                            DANGLING_COMMENT_GROUP: {
                                COMMENT: "# dangling comment group 2",
                                LINE_BREAK: "\n",
                                WHITESPACE: "    ",
                                COMMENT: "# dangling comment group 2"
                            },
                            LINE_BREAK: "\n",
                            LINE_BREAK: "\n",
                            LINE_BREAK: "\n",
                            WHITESPACE: "    ",
                            DANGLING_COMMENT_GROUP: {
                                COMMENT: "# dangling comment group 3",
                                LINE_BREAK: "\n",
                                WHITESPACE: "    ",
                                COMMENT: "# dangling comment group 3"
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
                            KEY_VALUE_WITH_COMMA_GROUP: {
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
                                COMMA: {
                                    COMMA: ","
                                },
                                KEY_VALUE: {
                                    LINE_BREAK: "\n",
                                    WHITESPACE: "    ",
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
                                },
                                COMMA: {
                                    COMMA: ","
                                }
                            },
                            LINE_BREAK: "\n",
                            WHITESPACE: "    ",
                            DANGLING_COMMENT_GROUP: {
                                COMMENT: "# dangling comment group 1",
                                LINE_BREAK: "\n",
                                WHITESPACE: "    ",
                                COMMENT: "# dangling comment group 1"
                            },
                            LINE_BREAK: "\n",
                            LINE_BREAK: "\n",
                            WHITESPACE: "    ",
                            DANGLING_COMMENT_GROUP: {
                                COMMENT: "# dangling comment group 2",
                                LINE_BREAK: "\n",
                                WHITESPACE: "    ",
                                COMMENT: "# dangling comment group 2"
                            },
                            LINE_BREAK: "\n",
                            LINE_BREAK: "\n",
                            WHITESPACE: "    ",
                            KEY_VALUE_WITH_COMMA_GROUP: {
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
                                COMMA: {
                                    COMMA: ","
                                },
                                KEY_VALUE: {
                                    LINE_BREAK: "\n",
                                    WHITESPACE: "    ",
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
                                },
                                COMMA: {
                                    COMMA: ","
                                }
                            },
                            LINE_BREAK: "\n",
                            LINE_BREAK: "\n",
                            WHITESPACE: "    ",
                            KEY_VALUE_WITH_COMMA_GROUP: {
                                KEY_VALUE: {
                                    COMMENT: "# leading comment 1",
                                    LINE_BREAK: "\n",
                                    WHITESPACE: "    ",
                                    COMMENT: "# leading comment 1",
                                    LINE_BREAK: "\n",
                                    WHITESPACE: "    ",
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
                                COMMA: {
                                    COMMA: ","
                                },
                                KEY_VALUE: {
                                    LINE_BREAK: "\n",
                                    WHITESPACE: "    ",
                                    COMMENT: "# leading comment 2",
                                    LINE_BREAK: "\n",
                                    WHITESPACE: "    ",
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
                                },
                                COMMA: {
                                    COMMA: ","
                                }
                            },
                            LINE_BREAK: "\n",
                            LINE_BREAK: "\n",
                            WHITESPACE: "    ",
                            DANGLING_COMMENT_GROUP: {
                                COMMENT: "# dangling comment group 3",
                                LINE_BREAK: "\n",
                                WHITESPACE: "    ",
                                COMMENT: "# dangling comment group 3"
                            },
                            LINE_BREAK: "\n",
                            BRACE_END: "}"
                        }
                    }
                }
            }
        )
    }
}
