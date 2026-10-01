mod error;

use crate::{ArrayNode, BoolNode, NullNode, NumberNode, ObjectNode, StringNode, ValueNode};
pub use error::Error;
use std::sync::Arc;
use tombi_json_lexer::{Lexer, Token};
use tombi_json_syntax::{SyntaxKind, T};
use tombi_json_value::Number;

use tombi_text::Span;

/// Maximum nesting depth for arrays and objects.
///
/// Bounds recursion in `parse_value`/`parse_array`/`parse_object` so a deeply
/// nested (but finite) document cannot exhaust the thread stack and abort the
/// process. Matches `serde_json`'s default recursion limit.
const MAX_RECURSION_DEPTH: usize = 128;

/// Parser for JSON documents
pub struct Parser<'a> {
    source: &'a str,
    /// Yields the non-trivia tokens on demand, so that they are not collected first.
    lexer: Lexer<'a>,
    /// The current non-trivia token.
    current: Token,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut lexer = Lexer::new(source);
        let current = lexer.next_token();
        Self {
            source,
            lexer,
            current,
        }
    }

    pub fn parse(&mut self) -> Result<ValueNode, crate::parser::Error> {
        let root = self.parse_value(0)?;

        // Ensure all tokens have been consumed
        if self.peek_kind() != SyntaxKind::EOF {
            return Err(Error::UnexpectedToken {
                expected: SyntaxKind::EOF,
                actual: self.peek_kind(),
            });
        }

        Ok(root)
    }

    fn parse_string(&mut self) -> Result<StringNode, crate::parser::Error> {
        // Get the current token (without advancing the position)
        match self.peek() {
            token if token.kind() == SyntaxKind::STRING => {
                let span = token.span();
                let contains_escape = token.contains_escape();
                // Get the string and advance the position
                let raw_str = &self.source[span.start.into()..span.end.into()];
                self.advance();

                // Remove the quotation marks
                let content = &raw_str[1..raw_str.len() - 1];

                if !contains_escape {
                    return Ok(StringNode {
                        value: content.to_owned(),
                        span,
                    });
                }

                Ok(StringNode {
                    value: unescape(content)?,
                    span,
                })
            }
            token => Err(Error::UnexpectedToken {
                expected: SyntaxKind::STRING,
                actual: token.kind(),
            }),
        }
    }

    fn parse_value(&mut self, depth: usize) -> Result<ValueNode, crate::parser::Error> {
        let token = self.peek();
        match token.kind() {
            SyntaxKind::STRING => self.parse_string().map(ValueNode::String),
            SyntaxKind::NUMBER => {
                let span = token.span();
                let num_str = &self.source[span.start.into()..span.end.into()];
                self.advance();

                // Parse as f64
                match num_str.parse::<f64>() {
                    Ok(n) => {
                        let num = if n.is_nan() || n.is_infinite() {
                            // Fallback for NaN or infinity
                            if num_str.starts_with('-') {
                                Number::from(-0.0)
                            } else {
                                Number::from(0.0)
                            }
                        } else {
                            Number::from_f64(n)
                        };

                        Ok(ValueNode::Number(NumberNode { value: num, span }))
                    }
                    Err(_) => Err(Error::InvalidValue),
                }
            }
            SyntaxKind::NULL => {
                let span = token.span();
                self.advance();
                Ok(ValueNode::Null(NullNode { span }))
            }
            SyntaxKind::BOOLEAN => {
                let span = token.span();
                let bool_str = &self.source[span.start.into()..span.end.into()];
                let value = bool_str == "true";
                self.advance();

                Ok(ValueNode::Bool(BoolNode { value, span }))
            }
            T!['['] => self.parse_array(depth),
            T!['{'] => self.parse_object(depth),
            _ => Err(Error::InvalidValue),
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<ValueNode, crate::parser::Error> {
        if depth >= MAX_RECURSION_DEPTH {
            return Err(Error::RecursionLimitExceeded {
                limit: MAX_RECURSION_DEPTH,
            });
        }

        // Consume the opening bracket
        let open_token = self.expect(T!['['])?;
        let start = open_token.span().start;
        let mut items = Vec::new();

        // Check if the array is empty
        if self.peek_kind() == T![']'] {
            let close_token = self.advance();
            let span = Span::new(start, close_token.span().end);
            return Ok(ValueNode::Array(ArrayNode {
                items: Default::default(),
                span,
            }));
        }

        // Parse array elements
        loop {
            // Parse value
            let value = self.parse_value(depth + 1)?;
            items.push(value);

            // Check for comma or closing bracket
            match self.peek_kind() {
                T![,] => {
                    self.advance(); // Consume comma
                }
                T![']'] => {
                    let close_token = self.advance();
                    let span = Span::new(start, close_token.span().end);

                    let array_node = ArrayNode {
                        items: Arc::new(items),
                        span,
                    };

                    return Ok(ValueNode::Array(array_node));
                }
                _ => {
                    return Err(Error::UnexpectedToken {
                        expected: T![']'],
                        actual: self.peek_kind(),
                    });
                }
            }

            // Check if we've reached the end of the array
            if self.peek_kind() == T![']'] {
                let close_token = self.advance();
                let span = Span::new(start, close_token.span().end);

                let array_node = ArrayNode {
                    items: Arc::new(items),
                    span,
                };

                return Ok(ValueNode::Array(array_node));
            }
        }
    }

    fn parse_object(&mut self, depth: usize) -> Result<ValueNode, crate::parser::Error> {
        if depth >= MAX_RECURSION_DEPTH {
            return Err(Error::RecursionLimitExceeded {
                limit: MAX_RECURSION_DEPTH,
            });
        }

        // Consume the opening brace
        let open_token = self.expect(T!['{'])?;
        let start = open_token.span().start;
        let mut properties: tombi_json_value::Map<StringNode, ValueNode> =
            tombi_json_value::Map::new();

        // Check if the object is empty
        if self.peek_kind() == T!['}'] {
            let close_token = self.advance();
            let span = Span::new(start, close_token.span().end);
            return Ok(ValueNode::Object(ObjectNode {
                properties: Default::default(),
                span,
            }));
        }

        // Parse object members
        loop {
            // Parse key (must be a string)
            let token = self.peek();
            if token.kind() != SyntaxKind::STRING {
                return Err(Error::UnexpectedToken {
                    expected: SyntaxKind::STRING,
                    actual: token.kind(),
                });
            }

            let key = self.parse_string()?;

            let vacant_entry = match properties.as_inner_mut().entry(key) {
                tombi_hashmap::map::Entry::Occupied(entry) => {
                    return Err(Error::DuplicateKey(entry.key().value.clone()));
                }
                tombi_hashmap::map::Entry::Vacant(entry) => entry,
            };

            // Expect colon
            self.expect(T![:])?;

            // Parse value
            let value = self.parse_value(depth + 1)?;

            // Store key and value without hashing the key a second time.
            vacant_entry.insert(value);

            // Check for comma or closing brace
            match self.peek_kind() {
                T![,] => {
                    self.advance(); // Consume comma
                }
                T!['}'] => {
                    let close_token = self.advance();
                    let span = Span::new(start, close_token.span().end);

                    let object_node = ObjectNode {
                        properties: Arc::new(properties),
                        span,
                    };

                    return Ok(ValueNode::Object(object_node));
                }
                _ => {
                    return Err(Error::UnexpectedToken {
                        expected: T!['}'],
                        actual: self.peek_kind(),
                    });
                }
            }

            // Check if we've reached the end of the object
            if self.peek_kind() == T!['}'] {
                let close_token = self.advance();
                let span = Span::new(start, close_token.span().end);

                let object_node = ObjectNode {
                    properties: Arc::new(properties),
                    span,
                };

                return Ok(ValueNode::Object(object_node));
            }
        }
    }

    fn peek(&self) -> Token {
        self.current
    }

    fn peek_kind(&self) -> SyntaxKind {
        self.current.kind()
    }

    fn advance(&mut self) -> Token {
        let token = self.current;
        self.current = self.lexer.next_token();
        token
    }

    fn expect(&mut self, kind: SyntaxKind) -> Result<Token, crate::parser::Error> {
        if self.peek_kind() == kind {
            Ok(self.advance())
        } else {
            Err(Error::UnexpectedToken {
                expected: kind,
                actual: self.peek_kind(),
            })
        }
    }
}

/// Decodes the escape sequences in the content of a string token.
///
/// The parts between escape sequences are copied as slices, not char by char.
fn unescape(content: &str) -> Result<String, crate::parser::Error> {
    let mut unescaped = String::with_capacity(content.len());
    let mut rest = content;
    while let Some(index) = rest.find('\\') {
        unescaped.push_str(&rest[..index]);
        let escape = &rest[index + 1..];
        let (c, len) = match escape.as_bytes().first() {
            Some(b'"') => ('"', 1),
            Some(b'\\') => ('\\', 1),
            Some(b'/') => ('/', 1),
            Some(b'b') => ('\u{0008}', 1),
            Some(b'f') => ('\u{000C}', 1),
            Some(b'n') => ('\n', 1),
            Some(b'r') => ('\r', 1),
            Some(b't') => ('\t', 1),
            Some(b'u') => unescape_unicode(&escape[1..]).map(|(c, len)| (c, len + 1))?,
            _ => return Err(Error::InvalidEscapeSequence),
        };
        unescaped.push(c);
        rest = &escape[len..];
    }
    unescaped.push_str(rest);
    Ok(unescaped)
}

/// Decodes the `XXXX` of a `\\uXXXX` escape, with the low surrogate that follows a high surrogate.
///
/// Returns the char and the length of the consumed input.
fn unescape_unicode(input: &str) -> Result<(char, usize), crate::parser::Error> {
    let code_point = hex4(input)?;
    match code_point {
        // High surrogate - expect low surrogate to follow
        0xD800..=0xDBFF => {
            let Some(low_surrogate) = input[4..].strip_prefix("\\u") else {
                return Err(Error::InvalidUnicodeCodePoint);
            };
            let low_surrogate = hex4(low_surrogate)?;
            if !(0xDC00..=0xDFFF).contains(&low_surrogate) {
                return Err(Error::InvalidUnicodeCodePoint);
            }
            let unicode_code_point =
                0x10000 + ((code_point - 0xD800) << 10) + (low_surrogate - 0xDC00);
            std::char::from_u32(unicode_code_point)
                .map(|c| (c, 10))
                .ok_or(Error::InvalidUnicodeCodePoint)
        }
        // Low surrogate without high surrogate
        0xDC00..=0xDFFF => Err(Error::InvalidUnicodeCodePoint),
        _ => std::char::from_u32(code_point)
            .map(|c| (c, 4))
            .ok_or(Error::InvalidUnicodeCodePoint),
    }
}

/// Parses the 4 hex digits at the start of `input`.
fn hex4(input: &str) -> Result<u32, crate::parser::Error> {
    match input.get(..4) {
        Some(digits) if digits.bytes().all(|digit| digit.is_ascii_hexdigit()) => {
            u32::from_str_radix(digits, 16).map_err(|_| Error::InvalidUnicodeEscape)
        }
        _ => Err(Error::InvalidUnicodeEscape),
    }
}

/// Parse a JSON string into a Tree
pub fn parse(source: &str) -> Result<ValueNode, crate::parser::Error> {
    let mut parser = Parser::new(source);
    parser.parse()
}

/// Parse a JSON string into a [`Document`][crate::Document], with the line index of `source`.
///
/// The line index is built from the line starts recorded while lexing, without scanning `source` again.
pub fn parse_document(
    source: impl Into<Box<str>>,
) -> Result<crate::Document, crate::parser::Error> {
    let text: Box<str> = source.into();
    let mut parser = Parser::new(&text);
    let value = parser.parse()?;
    // A successful parse has lexed up to the end of the source.
    let line_starts = parser.lexer.into_line_starts();
    Ok(crate::Document {
        value,
        line_index: std::sync::Arc::new(tombi_text::OwnedLineIndex::from_line_starts(
            text,
            line_starts,
        )),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_json_parser {
        ($(#[$meta:meta])* $name:ident, $source:expr, |$result:ident| $assertion:block) => {
            $(#[$meta])*
            #[test]
            fn $name() {
                let $result = parse(&$source);
                assert!($assertion);
            }
        };
    }

    #[test]
    fn test_parse_null() {
        let source = "null";
        let value_node = parse(source).unwrap();
        assert!(value_node.is_null());
    }

    #[test]
    fn test_parse_boolean() {
        let source = "true";
        let value_node = parse(source).unwrap();
        assert!(value_node.is_bool());
        pretty_assertions::assert_eq!(value_node.as_bool(), Some(true));

        let source = "false";
        let value_node = parse(source).unwrap();
        assert!(value_node.is_bool());
        pretty_assertions::assert_eq!(value_node.as_bool(), Some(false));
    }

    #[test]
    fn test_parse_number() {
        let source = "42";
        let value_node = parse(source).unwrap();
        assert!(value_node.is_number());
        pretty_assertions::assert_eq!(value_node.as_f64(), Some(42.0));

        let source = "-3.02";
        let value_node = parse(source).unwrap();
        assert!(value_node.is_number());
        pretty_assertions::assert_eq!(value_node.as_f64(), Some(-3.02));
    }

    #[test]
    fn test_parse_string() {
        let source = r#""hello""#;
        let value_node = parse(source).unwrap();
        assert!(value_node.is_string());
        pretty_assertions::assert_eq!(value_node.as_str(), Some("hello"));
    }

    test_json_parser!(
        long_unescaped_string_uses_copy_path,
        r#""abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ""#,
        |result| {
            result.as_ref().ok().and_then(ValueNode::as_str)
                == Some("abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ")
        }
    );

    test_json_parser!(
        escaped_string_preserves_decode_path,
        r#""abcdefghijklmnopqrstuvwxyz0123456789\nend""#,
        |result| {
            result.as_ref().ok().and_then(ValueNode::as_str)
                == Some("abcdefghijklmnopqrstuvwxyz0123456789\nend")
        }
    );

    test_json_parser!(
        unescape_copies_text_between_escapes,
        r#""a\"b\\c\/d\be\ff\ng\rh\ti\u00e9j\uD83E\uDD85k""#,
        |result| {
            result.as_ref().ok().and_then(ValueNode::as_str)
                == Some("a\"b\\c/d\u{0008}e\u{000C}f\ng\rh\ti\u{00e9}j\u{1F985}k")
        }
    );

    test_json_parser!(unescape_keeps_non_ascii_text, r#""日本\n語""#, |result| {
        result.as_ref().ok().and_then(ValueNode::as_str) == Some("日本\n語")
    });

    test_json_parser!(
        unescape_rejects_lone_low_surrogate,
        r#""\uDC00""#,
        |result| { matches!(result, Err(Error::InvalidUnicodeCodePoint)) }
    );

    test_json_parser!(
        unescape_rejects_high_surrogate_without_low_surrogate,
        r#""\uD83Ex""#,
        |result| { matches!(result, Err(Error::InvalidUnicodeCodePoint)) }
    );

    test_json_parser!(
        unescape_rejects_invalid_low_surrogate,
        r#""\uD83E\u0041""#,
        |result| { matches!(result, Err(Error::InvalidUnicodeCodePoint)) }
    );

    test_json_parser!(
        accepts_all_json_whitespace_characters,
        " \t\n\r[1,\n\r2]\r\n",
        |result| { result.is_ok() }
    );

    #[test]
    fn test_parse_array() {
        let source = "[1, 2, 3]";
        let value_node = parse(source).unwrap();
        assert!(value_node.is_array());

        let source = "[]";
        let value_node = parse(source).unwrap();
        assert!(value_node.is_array());
    }

    #[test]
    fn test_parse_object() {
        let source = r#"{"a": 1, "b": 2}"#;
        let value_node = parse(source).unwrap();
        assert!(value_node.is_object());

        let source = "{}";
        let value_node = parse(source).unwrap();
        assert!(value_node.is_object());
    }

    #[test]
    fn test_parse_complex() {
        let source = r#"
        {
            "name": "John",
            "age": 30,
            "isStudent": false,
            "courses": ["Math", "Physics"],
            "address": {
                "city": "New York",
                "zip": "10001"
            }
        }
        "#;

        let value_node = parse(source).unwrap();
        assert!(value_node.is_object());
    }

    test_json_parser!(
        nesting_at_limit_is_accepted,
        format!("{}1{}", "[".repeat(128), "]".repeat(128)),
        |result| { result.is_ok() }
    );

    test_json_parser!(
        nesting_beyond_limit_is_rejected,
        format!("{}1{}", "[".repeat(129), "]".repeat(129)),
        |result| { matches!(result, Err(Error::RecursionLimitExceeded { .. })) }
    );

    test_json_parser!(
        deeply_nested_arrays_do_not_overflow_stack,
        "[".repeat(500_000),
        |result| { result.is_err() }
    );

    test_json_parser!(
        deeply_nested_objects_do_not_overflow_stack,
        "{\"a\":".repeat(500_000),
        |result| { result.is_err() }
    );
}
