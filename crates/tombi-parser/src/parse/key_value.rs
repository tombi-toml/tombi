use tombi_syntax::{SyntaxKind::*, T};

use super::{Parse, TS_LINE_END};
use crate::{
    ErrorKind::*,
    parser::Parser,
    support::{leading_comments, trailing_comment},
};

impl Parse for tombi_ast::KeyValue {
    fn parse(p: &mut Parser) {
        let m = p.start();

        leading_comments(p);

        tombi_ast::Keys::parse(p);

        if !p.eat(T![=]) {
            p.error(crate::Error::new(ExpectedEqual, p.current_range()));
        }

        if p.at_ts(TS_LINE_END) {
            p.invalid_token();
            p.error(crate::Error::new(ExpectedValue, p.current_range()));
        } else if p.at(COMMENT) {
            p.invalid_token();
            p.error(crate::Error::new(ExpectedValue, p.previous_range()));
        } else {
            tombi_ast::Value::parse(p);
        }

        trailing_comment(p);

        m.complete(p, KEY_VALUE);
    }
}

#[cfg(test)]
mod test {
    use crate::{ErrorKind::*, test_parser};

    test_parser! {
        #[test]
        fn only_key("key1") -> Err([
            SyntaxError(ExpectedEqual, 0:4..0:4),
            SyntaxError(ExpectedValue, 0:4..0:4),
        ])
    }

    test_parser! {
        #[test]
        fn value_not_found("key1 = # INVALID") -> Err([
            SyntaxError(ExpectedValue, 0:5..0:6),
        ])
    }

    test_parser! {
        #[test]
        fn invalid_value("key1 = 2024-01-00T") -> Err([
            SyntaxError(InvalidLocalDateTime, 0:7..0:18),
            SyntaxError(ExpectedValue, 0:7..0:18),
        ])
    }

    test_parser! {
        #[test]
        fn value_not_found_in_multi_key_value(
            r#"
            key1 = 1
            key2 = # INVALID
            key3 = 3
            "#
        ) -> Err([
            SyntaxError(ExpectedValue, 1:5..1:6),
        ])
    }

    test_parser! {
        #[test]
        fn basic_string_without_begin_quote(
            r#"
            key1 = "str"
            key2 = invalid"
            key3 = 1
            "#
        ) -> Err([
            SyntaxError(InvalidKey, 1:7..1:15),
            SyntaxError(ExpectedValue, 1:7..1:15),
        ])
    }

    test_parser! {
        #[test]
        fn basic_string_without_end_quote(
            r#"
            key1 = "str"
            key2 = "invalid
            key3 = 1
            "#
        ) -> Err([
            SyntaxError(InvalidBasicString, 1:7..1:15),
            SyntaxError(ExpectedValue, 1:7..1:15),
        ])
    }

    test_parser! {
        #[test]
        fn literal_string_without_start_quote(
            r#"
            key1 = 'str'
            key2 = invalid'
            key3 = 1
            "#
        ) -> Err([
            SyntaxError(InvalidKey, 1:7..1:15),
            SyntaxError(ExpectedValue, 1:7..1:15),
        ])
    }

    test_parser! {
        #[test]
        fn literal_string_without_end_quote(
            r#"
            key1 = 'str'
            key2 = 'invalid
            key3 = 1
            "#
        ) -> Err([
            SyntaxError(InvalidLiteralString, 1:7..1:15),
            SyntaxError(ExpectedValue, 1:7..1:15),
        ])
    }

    test_parser! {
        #[test]
        fn without_equal(
            r#"
            key1 "value"
            key2 = 1
            "#
        ) -> Err([
            SyntaxError(ExpectedEqual, 0:5..0:12),
        ])
    }

    test_parser! {
        #[test]
        fn without_equal_on_root_item_with_comment(
            r#"
            key value # comment

            [aaa]
            key1 = 1
            "#
        ) -> Err([
            SyntaxError(ExpectedEqual, 0:4..0:9),
            SyntaxError(ExpectedValue, 0:4..0:9),
        ])
    }

    test_parser! {
        #[test]
        fn without_equal_on_root_item(
            r#"
            key value

            [aaa]
            key1 = 1
            "#
        ) -> Err([
            SyntaxError(ExpectedEqual, 0:4..0:9),
            SyntaxError(ExpectedValue, 0:4..0:9),
        ])
    }

    test_parser! {
        #[test]
        fn value_is_key(
            r#"
            key=value
            "#
        ) -> Err([
            SyntaxError(ExpectedValue, 0:4..0:9),
        ])
    }

    test_parser! {
        #[test]
        fn date_keys(
            r#"
            a.2001-02-08 = 7
            a.2001-02-09.2001-02-10 = 8
            2001-02-11.a.2001-02-12 = 9
            "#
        ) -> Ok(
            {
                KEY_VALUE_GROUP: {
                    KEY_VALUE: {
                        KEYS: {
                            BARE_KEY: {
                                BARE_KEY: "a"
                            },
                            DOT: ".",
                            BARE_KEY: {
                                BARE_KEY: "2001-02-08"
                            }
                        },
                        WHITESPACE: " ",
                        EQUAL: "=",
                        WHITESPACE: " ",
                        INTEGER_DEC: {
                            INTEGER_DEC: "7"
                        }
                    },
                    KEY_VALUE: {
                        LINE_BREAK: "\n",
                        KEYS: {
                            BARE_KEY: {
                                BARE_KEY: "a"
                            },
                            DOT: ".",
                            BARE_KEY: {
                                BARE_KEY: "2001-02-09"
                            },
                            DOT: ".",
                            BARE_KEY: {
                                BARE_KEY: "2001-02-10"
                            }
                        },
                        WHITESPACE: " ",
                        EQUAL: "=",
                        WHITESPACE: " ",
                        INTEGER_DEC: {
                            INTEGER_DEC: "8"
                        }
                    },
                    KEY_VALUE: {
                        LINE_BREAK: "\n",
                        KEYS: {
                            BARE_KEY: {
                                BARE_KEY: "2001-02-11"
                            },
                            DOT: ".",
                            BARE_KEY: {
                                BARE_KEY: "a"
                            },
                            DOT: ".",
                            BARE_KEY: {
                                BARE_KEY: "2001-02-12"
                            }
                        },
                        WHITESPACE: " ",
                        EQUAL: "=",
                        WHITESPACE: " ",
                        INTEGER_DEC: {
                            INTEGER_DEC: "9"
                        }
                    }
                }
            }
        )
    }

    test_parser! {
        #[test]
        fn minus_number_keys(
            r#"
            -01   = true
            "#
        ) -> Ok(
            {
                KEY_VALUE_GROUP: {
                    KEY_VALUE: {
                        KEYS: {
                            BARE_KEY: {
                                BARE_KEY: "-01"
                            }
                        },
                        WHITESPACE: "   ",
                        EQUAL: "=",
                        WHITESPACE: " ",
                        BOOLEAN: {
                            BOOLEAN: "true"
                        }
                    }
                }
            }
        )
    }

    test_parser! {
        #[test]
        fn hex_like_bare_keys(
            r#"
            0x96f = "hex-like key"
            0xDEADBEEF = "another hex-like"
            a.0xABC = "dotted hex"
            "#
        ) -> Ok(
            {
                KEY_VALUE_GROUP: {
                    KEY_VALUE: {
                        KEYS: {
                            BARE_KEY: {
                                BARE_KEY: "0x96f"
                            }
                        },
                        WHITESPACE: " ",
                        EQUAL: "=",
                        WHITESPACE: " ",
                        BASIC_STRING: {
                            BASIC_STRING: "\"hex-like key\""
                        }
                    },
                    KEY_VALUE: {
                        LINE_BREAK: "\n",
                        KEYS: {
                            BARE_KEY: {
                                BARE_KEY: "0xDEADBEEF"
                            }
                        },
                        WHITESPACE: " ",
                        EQUAL: "=",
                        WHITESPACE: " ",
                        BASIC_STRING: {
                            BASIC_STRING: "\"another hex-like\""
                        }
                    },
                    KEY_VALUE: {
                        LINE_BREAK: "\n",
                        KEYS: {
                            BARE_KEY: {
                                BARE_KEY: "a"
                            },
                            DOT: ".",
                            BARE_KEY: {
                                BARE_KEY: "0xABC"
                            }
                        },
                        WHITESPACE: " ",
                        EQUAL: "=",
                        WHITESPACE: " ",
                        BASIC_STRING: {
                            BASIC_STRING: "\"dotted hex\""
                        }
                    }
                }
            }
        )
    }

    test_parser! {
        #[test]
        fn octal_like_bare_keys(
            r#"
            0o755 = "octal-like key"
            0o777.permissions = "dotted octal"
            "#
        ) -> Ok(
            {
                KEY_VALUE_GROUP: {
                    KEY_VALUE: {
                        KEYS: {
                            BARE_KEY: {
                                BARE_KEY: "0o755"
                            }
                        },
                        WHITESPACE: " ",
                        EQUAL: "=",
                        WHITESPACE: " ",
                        BASIC_STRING: {
                            BASIC_STRING: "\"octal-like key\""
                        }
                    },
                    KEY_VALUE: {
                        LINE_BREAK: "\n",
                        KEYS: {
                            BARE_KEY: {
                                BARE_KEY: "0o777"
                            },
                            DOT: ".",
                            BARE_KEY: {
                                BARE_KEY: "permissions"
                            }
                        },
                        WHITESPACE: " ",
                        EQUAL: "=",
                        WHITESPACE: " ",
                        BASIC_STRING: {
                            BASIC_STRING: "\"dotted octal\""
                        }
                    }
                }
            }
        )
    }

    test_parser! {
        #[test]
        fn binary_like_bare_keys(
            r#"
            0b1010 = "binary-like key"
            0b11.0b00 = "dotted binary"
            "#
        ) -> Ok(
            {
                KEY_VALUE_GROUP: {
                    KEY_VALUE: {
                        KEYS: {
                            BARE_KEY: {
                                BARE_KEY: "0b1010"
                            }
                        },
                        WHITESPACE: " ",
                        EQUAL: "=",
                        WHITESPACE: " ",
                        BASIC_STRING: {
                            BASIC_STRING: "\"binary-like key\""
                        }
                    },
                    KEY_VALUE: {
                        LINE_BREAK: "\n",
                        KEYS: {
                            BARE_KEY: {
                                BARE_KEY: "0b11"
                            },
                            DOT: ".",
                            BARE_KEY: {
                                BARE_KEY: "0b00"
                            }
                        },
                        WHITESPACE: " ",
                        EQUAL: "=",
                        WHITESPACE: " ",
                        BASIC_STRING: {
                            BASIC_STRING: "\"dotted binary\""
                        }
                    }
                }
            }
        )
    }

    test_parser! {
        #[test]
        fn hex_key_with_hex_value(
            r#"
            0x96f = 0x96f
            "#
        ) -> Ok(
            {
                KEY_VALUE_GROUP: {
                    KEY_VALUE: {
                        KEYS: {
                            BARE_KEY: {
                                BARE_KEY: "0x96f"
                            }
                        },
                        WHITESPACE: " ",
                        EQUAL: "=",
                        WHITESPACE: " ",
                        INTEGER_HEX: {
                            INTEGER_HEX: "0x96f"
                        }
                    }
                }
            }
        )
    }

    test_parser! {
        #[test]
        fn inline_table_with_hex_key(
            r#"
            table = { 0x96f = "value" }
            "#
        ) -> Ok(
            {
                KEY_VALUE_GROUP: {
                    KEY_VALUE: {
                        KEYS: {
                            BARE_KEY: {
                                BARE_KEY: "table"
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
                                            BARE_KEY: "0x96f"
                                        }
                                    },
                                    WHITESPACE: " ",
                                    EQUAL: "=",
                                    WHITESPACE: " ",
                                    BASIC_STRING: {
                                        BASIC_STRING: "\"value\""
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
}
