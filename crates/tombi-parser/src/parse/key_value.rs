use tombi_syntax::{SyntaxKind::*, T};

use super::{leading_comments, trailing_comment, Parse, TS_LINE_END};
use crate::{parser::Parser, ErrorKind::*};

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
    use crate::{test_parser, ErrorKind::*};

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
}
