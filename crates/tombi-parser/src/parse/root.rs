use tombi_ast_syntax::{SyntaxKind::*, T};

use super::{Parse, TS_LINE_END, invalid_line};
use crate::{
    ErrorKind::*,
    parser::Parser,
    support::{leading_comments, peek_leading_comments, trailing_comment},
    token_set::TS_NEXT_SECTION,
};

impl Parse for tombi_ast_syntax::Root {
    fn parse(p: &mut Parser<'_>) {
        let m = p.start();

        loop {
            while p.eat(LINE_BREAK) {}

            Vec::<tombi_ast_syntax::DanglingCommentGroup>::parse(p);

            let n = peek_leading_comments(p);
            if p.nth_at_ts(n, TS_NEXT_SECTION) {
                break;
            }

            tombi_ast_syntax::KeyValueGroup::parse(p);

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
                tombi_ast_syntax::ArrayOfTable::parse(p);
            } else if p.nth_at(n, T!['[']) {
                tombi_ast_syntax::Table::parse(p);
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
        fn preserves_grapheme_columns_after_combining_character(
            "\"e\u{301}\" = true"
        ) -> Ok(|root| -> {
            let value = root.key_values().next().unwrap().value().unwrap();
            value.range()
                == tombi_text::Range::new(
                    tombi_text::Position::new(0, 6),
                    tombi_text::Position::new(0, 10),
                )
                && matches!(
                    root.nodes_at_position(tombi_text::Position::new(0, 6)).next(),
                    Some(tombi_ast_syntax::TomlNode::Boolean(_))
                )
        })
    }

    test_parser! {
        #[test]
        fn preserves_grapheme_columns_after_zwj_emoji(
            "\"👨‍👩‍👧‍👦\" = true"
        ) -> Ok(|root| -> {
            let value = root.key_values().next().unwrap().value().unwrap();
            value.range()
                == tombi_text::Range::new(
                    tombi_text::Position::new(0, 6),
                    tombi_text::Position::new(0, 10),
                )
                && matches!(
                    root.nodes_at_position(tombi_text::Position::new(0, 6)).next(),
                    Some(tombi_ast_syntax::TomlNode::Boolean(_))
                )
        })
    }

    test_parser! {
        #[test]
        fn preserves_grapheme_columns_after_unicode_checkpoint(
            &format!("\"{}\" = true", "é".repeat(80))
        ) -> Ok(|root| -> {
            let value = root.key_values().next().unwrap().value().unwrap();
            value.range()
                == tombi_text::Range::new(
                    tombi_text::Position::new(0, 85),
                    tombi_text::Position::new(0, 89),
                )
                && matches!(
                    root.nodes_at_position(tombi_text::Position::new(0, 85)).next(),
                    Some(tombi_ast_syntax::TomlNode::Boolean(_))
                )
        })
    }

    test_parser! {
        #[test]
        fn resolves_end_of_64_grapheme_unicode_line(
            &format!("é = \"{}\"", "a".repeat(58))
        ) -> Ok(|root| -> {
            root.key_values().next().unwrap().value().unwrap().range().end
                == tombi_text::Position::new(0, 64)
                && matches!(
                    root.nodes_at_position(tombi_text::Position::new(0, 64)).next(),
                    Some(tombi_ast_syntax::TomlNode::BasicString(_))
                )
        })
    }

    test_parser! {
        #[test]
        fn resolves_end_of_128_grapheme_unicode_line(
            &format!("é = \"{}\"", "a".repeat(122))
        ) -> Ok(|root| -> {
            root.key_values().next().unwrap().value().unwrap().range().end
                == tombi_text::Position::new(0, 128)
                && matches!(
                    root.nodes_at_position(tombi_text::Position::new(0, 128)).next(),
                    Some(tombi_ast_syntax::TomlNode::BasicString(_))
                )
        })
    }

    test_parser! {
        #[test]
        fn root_items_include_top_level_key_values(
            "a = 1\n[t]\nb = 2"
        ) -> Ok(|root| -> {
            let concrete: Vec<_> = root.items().map(|item| match item {
                tombi_ast_syntax::RootItem::KeyValue(_) => "key-value",
                tombi_ast_syntax::RootItem::Table(_) => "table",
                tombi_ast_syntax::RootItem::ArrayOfTable(_) => "array-of-table",
            }).collect();
            let public: Vec<_> = tombi_ast::RootNode::items(&root).map(|item| match item {
                tombi_ast_syntax::RootItem::KeyValue(_) => "key-value",
                tombi_ast_syntax::RootItem::Table(_) => "table",
                tombi_ast_syntax::RootItem::ArrayOfTable(_) => "array-of-table",
            }).collect();

            concrete == ["key-value", "table"] && public == concrete
        })
    }

    test_parser! {
        #[test]
        fn resolves_ascii_eof_after_standalone_carriage_return(
            "a\r"
        ) -> RawAssert(|parsed| {
            let root = parsed.root();
            !parsed.errors.is_empty()
                && root.nodes_at_position(root.range().end).next().is_some()
        })
    }

    test_parser! {
        #[test]
        fn resolves_unicode_eof_after_standalone_carriage_return(
            "é\r"
        ) -> RawAssert(|parsed| {
            let root = parsed.root();
            !parsed.errors.is_empty()
                && root.nodes_at_position(root.range().end).next().is_some()
        })
    }

    test_parser! {
        #[test]
        fn parses_root_dangling_comments_before_table(
            r#"
            # dangling_comment1
            # dangling_comment2

            # table leading comment1
            # table leading comment2
            [table]
            "#
        ) -> Ok(
            {
                DANGLING_COMMENT_GROUP: {
                    COMMENT: "# dangling_comment1",
                    LINE_BREAK: "\n",
                    COMMENT: "# dangling_comment2"
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
