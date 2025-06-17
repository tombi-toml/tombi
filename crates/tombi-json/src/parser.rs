mod error;

use crate::{ArrayNode, BoolNode, NullNode, NumberNode, ObjectNode, StringNode, ValueNode};
pub use error::Error;
use tombi_json_lexer::{lex, Lexed, Token};
use tombi_json_syntax::{SyntaxKind, T};
use tombi_json_value::Number;
use tombi_text::Range;

/// Parser for JSON documents
pub struct Parser<'a> {
    source: &'a str,
    lexed: Lexed,
    position: usize,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let lexed = lex(source);
        Self {
            source,
            lexed,
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Result<ValueNode, crate::parser::Error> {
        // Skip leading trivia
        while let Some(token) = self.peek() {
            if token.kind().is_trivia() {
                self.advance();
            } else {
                break;
            }
        }

        let root = self.parse_value()?;

        // Skip trailing trivia
        while let Some(token) = self.peek() {
            if token.kind().is_trivia() {
                self.advance();
            } else {
                break;
            }
        }

        // Ensure all tokens have been consumed
        if let Some(token) = self.peek() {
            if token.kind() != SyntaxKind::EOF {
                return Err(Error::UnexpectedToken {
                    expected: SyntaxKind::EOF,
                    actual: token.kind(),
                });
            }
        }

        Ok(root)
    }

    fn parse_string(&mut self) -> Result<StringNode, crate::parser::Error> {
        // Get the current token (without advancing the position)
        match self.peek() {
            Some(token) if token.kind() == SyntaxKind::STRING => {
                let span = token.span();
                let range = token.range();
                // Get the string and advance the position
                let raw_str = &self.source[span.start.into()..span.end.into()];
                self.advance();

                // Remove the quotation marks
                let content = &raw_str[1..raw_str.len() - 1];

                // Process the string including escape sequences
                let mut processed = String::with_capacity(content.len());
                let mut chars = content.chars().peekable();

                while let Some(c) = chars.next() {
                    if c == '\\' {
                        // Handle escape sequences
                        match chars.next() {
                            Some('"') => processed.push('"'),
                            Some('\\') => processed.push('\\'),
                            Some('/') => processed.push('/'),
                            Some('b') => processed.push('\u{0008}'),
                            Some('f') => processed.push('\u{000C}'),
                            Some('n') => processed.push('\n'),
                            Some('r') => processed.push('\r'),
                            Some('t') => processed.push('\t'),
                            Some('u') => {
                                // Unicode escape sequence: \uXXXX
                                let mut code_point = 0u32;
                                for _ in 0..4 {
                                    match chars.next() {
                                        Some(hex) if hex.is_ascii_hexdigit() => {
                                            code_point =
                                                code_point * 16 + hex.to_digit(16).unwrap();
                                        }
                                        _ => return Err(Error::InvalidUnicodeEscape),
                                    }
                                }

                                // Check for surrogate pairs
                                if (0xD800..=0xDBFF).contains(&code_point) {
                                    // High surrogate - expect low surrogate to follow
                                    if chars.next() == Some('\\') && chars.next() == Some('u') {
                                        let mut low_surrogate = 0u32;
                                        for _ in 0..4 {
                                            match chars.next() {
                                                Some(hex) if hex.is_ascii_hexdigit() => {
                                                    low_surrogate = low_surrogate * 16
                                                        + hex.to_digit(16).unwrap();
                                                }
                                                _ => return Err(Error::InvalidUnicodeEscape),
                                            }
                                        }

                                        if (0xDC00..=0xDFFF).contains(&low_surrogate) {
                                            // Valid surrogate pair - decode to actual Unicode code point
                                            let high = code_point - 0xD800;
                                            let low = low_surrogate - 0xDC00;
                                            let unicode_code_point = 0x10000 + (high << 10) + low;

                                            match std::char::from_u32(unicode_code_point) {
                                                Some(unicode_char) => processed.push(unicode_char),
                                                None => return Err(Error::InvalidUnicodeCodePoint),
                                            }
                                        } else {
                                            return Err(Error::InvalidUnicodeCodePoint);
                                        }
                                    } else {
                                        return Err(Error::InvalidUnicodeCodePoint);
                                    }
                                } else if (0xDC00..=0xDFFF).contains(&code_point) {
                                    // Low surrogate without high surrogate
                                    return Err(Error::InvalidUnicodeCodePoint);
                                } else {
                                    // Regular Unicode code point
                                    match std::char::from_u32(code_point) {
                                        Some(unicode_char) => processed.push(unicode_char),
                                        None => return Err(Error::InvalidUnicodeCodePoint),
                                    }
                                }
                            }
                            _ => return Err(Error::InvalidEscapeSequence),
                        }
                    } else {
                        processed.push(c);
                    }
                }

                Ok(StringNode {
                    value: processed,
                    range,
                })
            }
            Some(token) => Err(Error::UnexpectedToken {
                expected: SyntaxKind::STRING,
                actual: token.kind(),
            }),
            None => Err(Error::UnexpectedEof),
        }
    }

    fn parse_value(&mut self) -> Result<ValueNode, crate::parser::Error> {
        match self.peek() {
            Some(token) => {
                match token.kind() {
                    SyntaxKind::STRING => self.parse_string().map(ValueNode::String),
                    SyntaxKind::NUMBER => {
                        let token = self.peek().unwrap();
                        let span = token.span();
                        let range = token.range();
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

                                Ok(ValueNode::Number(NumberNode { value: num, range }))
                            }
                            Err(_) => Err(Error::InvalidValue),
                        }
                    }
                    SyntaxKind::NULL => {
                        let token = self.peek().unwrap();
                        let range = token.range();
                        self.advance();
                        Ok(ValueNode::Null(NullNode { range }))
                    }
                    SyntaxKind::BOOLEAN => {
                        let token = self.peek().unwrap();
                        let span = token.span();
                        let range = token.range();
                        let bool_str = &self.source[span.start.into()..span.end.into()];
                        let value = bool_str == "true";
                        self.advance();

                        Ok(ValueNode::Bool(BoolNode { value, range }))
                    }
                    T!['['] => self.parse_array(),
                    T!['{'] => self.parse_object(),
                    _ => Err(Error::InvalidValue),
                }
            }
            None => Err(Error::UnexpectedEof),
        }
    }

    fn parse_array(&mut self) -> Result<ValueNode, crate::parser::Error> {
        // Consume the opening bracket
        let open_token = self.expect(T!['['])?;
        let start_range = open_token.range();
        let mut items = Vec::new();

        // Check if the array is empty
        if let Some(token) = self.peek() {
            if token.kind() == T![']'] {
                let close_token = self.advance().unwrap();
                let full_range = Range::new(start_range.start, close_token.range().end);
                return Ok(ValueNode::Array(ArrayNode {
                    items: Vec::new(),
                    range: full_range,
                }));
            }
        }

        // Parse array elements
        loop {
            // Parse value
            let value = self.parse_value()?;
            items.push(value);

            // Check for comma or closing bracket
            match self.peek_kind() {
                Some(T![,]) => {
                    self.advance(); // Consume comma
                }
                Some(T![']']) => {
                    let close_token = self.advance().unwrap();
                    let full_range = Range::new(start_range.start, close_token.range().end);

                    let array_node = ArrayNode {
                        items,
                        range: full_range,
                    };

                    return Ok(ValueNode::Array(array_node));
                }
                _ => {
                    return Err(Error::UnexpectedToken {
                        expected: T![']'],
                        actual: self.peek_kind().unwrap_or(SyntaxKind::EOF),
                    })
                }
            }

            // Check if we've reached the end of the array
            if let Some(token) = self.peek() {
                if token.kind() == T![']'] {
                    let close_token = self.advance().unwrap();
                    let full_range = Range::new(start_range.start, close_token.range().end);

                    let array_node = ArrayNode {
                        items,
                        range: full_range,
                    };

                    return Ok(ValueNode::Array(array_node));
                }
            }
        }
    }

    fn parse_object(&mut self) -> Result<ValueNode, crate::parser::Error> {
        // Consume the opening brace
        let open_token = self.expect(T!['{'])?;
        let start_range = open_token.range();
        let mut properties: tombi_json_value::Map<StringNode, ValueNode> =
            tombi_json_value::Map::new();

        // Check if the object is empty
        if let Some(token) = self.peek() {
            if token.kind() == T!['}'] {
                let close_token = self.advance().unwrap();
                let full_range = Range::new(start_range.start, close_token.range().end);
                return Ok(ValueNode::Object(ObjectNode {
                    properties: tombi_json_value::Map::new(),
                    range: full_range,
                }));
            }
        }

        // Parse object members
        loop {
            // Parse key (must be a string)
            let Some(token) = self.peek() else {
                return Err(Error::UnexpectedEof);
            };

            if token.kind() != SyntaxKind::STRING {
                return Err(Error::UnexpectedToken {
                    expected: SyntaxKind::STRING,
                    actual: token.kind(),
                });
            }

            let key = self.parse_string()?;

            // Check for duplicate keys
            if properties.contains_key(&key) {
                return Err(Error::DuplicateKey(key.value));
            }

            // Expect colon
            self.expect(T![:])?;

            // Parse value
            let value = self.parse_value()?;

            // Store key and value
            properties.insert(key, value);

            // Check for comma or closing brace
            match self.peek_kind() {
                Some(T![,]) => {
                    self.advance(); // Consume comma
                }
                Some(T!['}']) => {
                    let close_token = self.advance().unwrap();
                    let full_range = Range::new(start_range.start, close_token.range().end);

                    let object_node = ObjectNode {
                        properties,
                        range: full_range,
                    };

                    return Ok(ValueNode::Object(object_node));
                }
                _ => {
                    return Err(Error::UnexpectedToken {
                        expected: T!['}'],
                        actual: self.peek_kind().unwrap_or(SyntaxKind::EOF),
                    })
                }
            }

            // Check if we've reached the end of the object
            if let Some(token) = self.peek() {
                if token.kind() == T!['}'] {
                    let close_token = self.advance().unwrap();
                    let full_range = Range::new(start_range.start, close_token.range().end);

                    let object_node = ObjectNode {
                        properties,
                        range: full_range,
                    };

                    return Ok(ValueNode::Object(object_node));
                }
            }
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.lexed.tokens.get(self.position)
    }

    fn peek_kind(&self) -> Option<SyntaxKind> {
        self.peek().map(|t| t.kind())
    }

    fn advance(&mut self) -> Option<&Token> {
        // Save the start position
        let position = self.position;
        let token = self.lexed.tokens.get(position);

        // Advance the position
        self.position += 1;

        // Skip trivia tokens
        while let Some(token) = self.lexed.tokens.get(self.position) {
            if token.kind().is_trivia() {
                self.position += 1;
            } else {
                break;
            }
        }

        token
    }

    fn expect(&mut self, kind: SyntaxKind) -> Result<&Token, crate::parser::Error> {
        match self.peek() {
            Some(token) if token.kind() == kind => Ok(self.advance().unwrap()),
            Some(token) => Err(Error::UnexpectedToken {
                expected: kind,
                actual: token.kind(),
            }),
            None => Err(Error::UnexpectedEof),
        }
    }
}

/// Parse a JSON string into a Tree
pub fn parse(source: &str) -> Result<ValueNode, crate::parser::Error> {
    let mut parser = Parser::new(source);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
